use anyhow::{Context, Result, anyhow, bail};
use reqwest::Client;
use serde::Deserialize;

use super::manifest::{GachaManifest, LEGACY_MANIFEST_FILE, MANIFEST_FILE};
use crate::fs_util::write_atomic;
use crate::http::{self, ResponseExt};
use crate::settings;

#[derive(Debug, Deserialize)]
struct IndexFile {
    schema_version: u32,
    gachas: Vec<String>,
}

const INDEX_SCHEMA_VERSION: u32 = 3;

pub async fn ensure_all_fetched() -> Result<u32> {
    let base = settings::get().assets.fetch_url.trim().to_string();
    if base.is_empty() {
        return Err(anyhow!("assets.fetch_url in settings.toml is empty"));
    }
    let root = format!("{}/gacha", base.trim_end_matches('/'));

    super::state::flatten_publisher_dirs_once();
    let client = http::client();
    let index = fetch_index(client, &root).await?;

    let mut written: u32 = 0;
    for game in &index.gachas {
        match fetch_one(client, &root, game).await {
            Ok(()) => written += 1,
            Err(e) => tracing::error!("{}: {:#}", game, e),
        }
    }
    Ok(written)
}

async fn get_text(client: &Client, url: &str) -> Result<String> {
    Ok(client.get(url).send().await?.check()?.text().await?)
}

async fn fetch_index(client: &Client, root: &str) -> Result<IndexFile> {
    let url = format!("{root}/index.toml");
    let parsed: IndexFile = toml::from_str(&get_text(client, &url).await?)
        .with_context(|| format!("invalid gacha index from {url}"))?;
    if parsed.schema_version != INDEX_SCHEMA_VERSION {
        bail!(
            "gacha index schema_version {} not supported (expected {})",
            parsed.schema_version,
            INDEX_SCHEMA_VERSION
        );
    }
    Ok(parsed)
}

async fn fetch_one(client: &Client, root: &str, game: &str) -> Result<()> {
    let url = format!("{root}/{game}/{MANIFEST_FILE}");
    let body = get_text(client, &url).await?;

    // validate before writing; dont drop broken manifests next to good ones pleaseee
    GachaManifest::parse(&body).with_context(|| format!("invalid manifest from {url}"))?;

    let dir = crate::gachas_dir().join(game);
    write_atomic(&dir.join(MANIFEST_FILE), &body)?;
    let _ = fs_err::remove_file(dir.join(LEGACY_MANIFEST_FILE));
    Ok(())
}
