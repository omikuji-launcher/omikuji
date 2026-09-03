use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use std::sync::LazyLock;

// one client so the connection pool is actually reused; per-call clients redo TLS every time
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!("omikuji/", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_default()
});

pub fn client() -> &'static reqwest::Client {
    &CLIENT
}

// on_percent fires at most once per whole percent, size_hint covers servers that send no content-length
pub async fn download_with_progress(
    url: &str,
    size_hint: u64,
    mut on_percent: impl FnMut(f64),
) -> Result<Vec<u8>> {
    let resp = client()
        .get(url)
        .send()
        .await?
        .error_for_status()
        .map_err(|e| anyhow!("download {}: {}", url, e))?;

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
