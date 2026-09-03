use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::time::{SystemTime, UNIX_EPOCH};

use super::EditionApi;

// the sign covers the exact bytes of head, so it is formatted not serialized
fn header(api: &EditionApi) -> String {
    use md5::{Digest, Md5};

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let head = format!(r#"{{"game_tag":"{}","time":{}}}"#, api.game_tag, time);
    let mut hasher = Md5::new();
    hasher.update(format!("{}{}", head, api.salt).as_bytes());
    format!(r#"{{"head":{},"sign":"{:x}"}}"#, head, hasher.finalize())
}

#[derive(Deserialize)]
struct Envelope<T> {
    code: i64,
    data: Option<T>,
}

pub async fn get<T: DeserializeOwned>(api: &EditionApi, url: &str) -> Result<T> {
    let resp = crate::http::client()
        .get(url)
        .header("Authorization", header(api))
        .send()
        .await
        .map_err(|e| anyhow!("GET {}: {}", url, e))?;
    if !resp.status().is_success() {
        anyhow::bail!("GET {}: http {}", url, resp.status());
    }
    let envelope: Envelope<T> = resp
        .json()
        .await
        .map_err(|e| anyhow!("parse {}: {}", url, e))?;
    if envelope.code != 200 {
        anyhow::bail!("{} returned code {}", url, envelope.code);
    }
    envelope
        .data
        .ok_or_else(|| anyhow!("{} returned no data", url))
}
