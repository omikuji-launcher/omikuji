use anyhow::{Result, anyhow};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IndexFile {
    schema_version: u32,
    gachas: Vec<String>,
}

const INDEX_SCHEMA_VERSION: u32 = 2;

pub async fn ensure_all_fetched() -> Result<u32> {
    let base = crate::settings::get().assets.fetch_url.trim().to_string();
    if base.is_empty() {
        return Err(anyhow!(
            "assets.fetch_url is empty in settings.toml — check [assets]"
        ));
    }

    super::state::flatten_publisher_dirs_once();
    let client = crate::http::client();
    let index = fetch_index(client, &base).await?;

    let mut written: u32 = 0;
    for game in &index.gachas {
        match fetch_one(client, &base, game).await {
            Ok(()) => written += 1,
            Err(e) => tracing::error!("{}: {}", game, e),
        }
    }
    Ok(written)
}

async fn fetch_index(client: &reqwest::Client, base: &str) -> Result<IndexFile> {
    let url = format!("{}/gacha/index.json", base.trim_end_matches('/'));
    let body = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let parsed: IndexFile = serde_json::from_slice(&body)
        .map_err(|e| anyhow!("invalid gacha index from {}: {}", url, e))?;
    if parsed.schema_version != INDEX_SCHEMA_VERSION {
        return Err(anyhow!(
            "gacha index schema_version {} not supported (expected {})",
            parsed.schema_version,
            INDEX_SCHEMA_VERSION
        ));
    }
    Ok(parsed)
}

async fn fetch_one(client: &reqwest::Client, base: &str, game: &str) -> Result<()> {
    let url = format!(
        "{}/gacha/{}/manifest.json",
        base.trim_end_matches('/'),
        game
    );
    let body = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    // validate before writing; dont drop broken json next to good ones pleaseee
    let _parsed: super::manifest::GachaManifest = serde_json::from_slice(&body)
        .map_err(|e| anyhow!("invalid manifest from {}: {}", url, e))?;

    let path = crate::gachas_dir().join(game).join("manifest.json");
    crate::fs_util::write_atomic(&path, &body)?;
    Ok(())
}
