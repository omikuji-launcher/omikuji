use anyhow::Result;
use chrono::{DateTime, Local};
use futures_util::StreamExt;
use reqwest::header::HeaderMap;
use reqwest::{Response, StatusCode, Url, blocking};
use std::fmt;
use std::sync::LazyLock;
use std::time::Duration;

pub const USER_AGENT: &str = concat!("omikuji/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub enum HttpError {
    RateLimited {
        host: String,
        resets_at: Option<DateTime<Local>>,
    },
    Status {
        host: String,
        status: StatusCode,
    },
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited {
                host,
                resets_at: Some(at),
            } => write!(
                f,
                "{host} rate limit reached, try again after {}",
                at.format("%H:%M")
            ),
            Self::RateLimited {
                host,
                resets_at: None,
            } => write!(f, "{host} rate limit reached, try again later"),
            Self::Status { host, status } => write!(f, "{host} answered {status}"),
        }
    }
}

impl std::error::Error for HttpError {}

pub trait ResponseExt: Sized {
    fn check(self) -> Result<Self, HttpError>;
}

impl ResponseExt for Response {
    fn check(self) -> Result<Self, HttpError> {
        classify(self.url(), self.status(), self.headers()).map_or(Ok(self), Err)
    }
}

impl ResponseExt for blocking::Response {
    fn check(self) -> Result<Self, HttpError> {
        classify(self.url(), self.status(), self.headers()).map_or(Ok(self), Err)
    }
}

fn classify(url: &Url, status: StatusCode, headers: &HeaderMap) -> Option<HttpError> {
    if !status.is_client_error() && !status.is_server_error() {
        return None;
    }
    let host = url.host_str().unwrap_or_default().to_string();
    tracing::warn!("{host}{}: {status}", url.path());
    let exhausted = header(headers, "x-ratelimit-remaining") == Some("0");
    if status == StatusCode::TOO_MANY_REQUESTS || (status == StatusCode::FORBIDDEN && exhausted) {
        return Some(HttpError::RateLimited {
            host,
            resets_at: rate_limit_reset(headers),
        });
    }
    Some(HttpError::Status { host, status })
}

fn rate_limit_reset(headers: &HeaderMap) -> Option<DateTime<Local>> {
    if let Some(epoch) = header(headers, "x-ratelimit-reset").and_then(|v| v.parse::<i64>().ok()) {
        return DateTime::from_timestamp(epoch, 0).map(|t| t.with_timezone(&Local));
    }
    let secs = header(headers, "retry-after")?.parse::<i64>().ok()?;
    Some(Local::now() + chrono::Duration::seconds(secs))
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

// one client so the connection pool is actually reused; per-call clients redo TLS every time
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap_or_default()
});

pub fn client() -> &'static reqwest::Client {
    &CLIENT
}

pub async fn with_retries<T, F, Fut>(attempts: u32, mut op: F) -> Result<T>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 1;
    loop {
        match op(attempt).await {
            Err(e) if attempt < attempts => {
                tracing::warn!("attempt {}/{} failed: {:#}", attempt, attempts, e);
                tokio::time::sleep(Duration::from_secs(attempt.into())).await;
                attempt += 1;
            }
            result => return result,
        }
    }
}

// signed cdn urls carry a query, never let it into a file name
pub fn url_file_name(url: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .ok()?
        .path_segments()?
        .next_back()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

// on_percent fires at most once per whole percent, size_hint covers servers that send no content-length
pub async fn download_with_progress(
    url: &str,
    size_hint: u64,
    mut on_percent: impl FnMut(f64),
) -> Result<Vec<u8>> {
    let resp = client().get(url).send().await?.check()?;

    let total = resp.content_length().unwrap_or(size_hint);
    let mut buf: Vec<u8> = if total > 0 {
        Vec::with_capacity(total as usize)
    } else {
        Vec::new()
    };

    let mut stream = resp.bytes_stream();
    let mut last_pct = -1.0_f64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buf.extend_from_slice(&chunk);
        if total > 0 {
            let pct = (buf.len() as f64 / total as f64) * 100.0;
            if pct - last_pct >= 1.0 {
                on_percent(pct);
                last_pct = pct;
            }
        }
    }
    Ok(buf)
}
