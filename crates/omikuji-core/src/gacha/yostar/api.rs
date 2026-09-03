use anyhow::{Result, anyhow};
use serde::Deserialize;

use super::{EditionApi, auth};
use crate::gacha::file_sync::{SyncFile, deserialize_size};
use crate::gacha::manifest::GachaManifest;
use crate::gacha::strategies::InstallSize;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub version: String,
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct FileIndex {
    pub base_url: String,
    pub files: Vec<IndexFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexFile {
    pub path: String,
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u64,
}

impl FileIndex {
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    pub fn sync_files(&self) -> Vec<SyncFile> {
        self.files
            .iter()
            .map(|f| SyncFile {
                rel_path: f.path.clone(),
                size: f.size,
                url: format!("{}{}", self.base_url, encode_path(&f.path)),
            })
            .collect()
    }
}

pub async fn fetch_game_config(api: &EditionApi) -> Result<GameConfig> {
    #[derive(Deserialize)]
    struct Raw {
        game_latest_version: String,
        game_latest_file_path: String,
    }

    let raw: Raw = auth::get(api, &format!("{}/api/launcher/game/config", api.api_url)).await?;
    if raw.game_latest_version.is_empty() {
        anyhow::bail!("launcher config returned an empty version");
    }
    Ok(GameConfig {
        version: raw.game_latest_version,
        file_path: raw.game_latest_file_path,
    })
}

pub async fn fetch_file_index(api: &EditionApi, config: &GameConfig) -> Result<FileIndex> {
    #[derive(Deserialize)]
    struct RawUrl {
        url: String,
    }

    #[derive(Deserialize)]
    struct RawIndex {
        source: String,
        file: Vec<IndexFile>,
    }

    let url = reqwest::Url::parse_with_params(
        &format!("{}/api/launcher/game/config/json", api.api_url),
        &[
            ("version", config.version.as_str()),
            ("file_path", config.file_path.as_str()),
        ],
    )?;
    let located: RawUrl = auth::get(api, url.as_str()).await?;

    let resp = crate::http::client()
        .get(&located.url)
        .send()
        .await
        .map_err(|e| anyhow!("GET {}: {}", located.url, e))?;
    if !resp.status().is_success() {
        anyhow::bail!("GET {}: http {}", located.url, resp.status());
    }
    let raw: RawIndex = resp
        .json()
        .await
        .map_err(|e| anyhow!("parse file index: {}", e))?;

    let source = if raw.source.starts_with('/') {
        raw.source
    } else {
        format!("/{}", raw.source)
    };
    Ok(FileIndex {
        base_url: format!("{}{}", api.cdn_url, source),
        files: raw.file,
    })
}

// asset names carry [ ] and # (hopefully thats all)
fn encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for b in path.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub async fn fetch_install_size(manifest: &GachaManifest, edition_id: &str) -> Result<InstallSize> {
    let api = super::edition_api(manifest, edition_id)?;
    let index = fetch_file_index(&api, &fetch_game_config(&api).await?).await?;
    let total = index.total_size();
    Ok(InstallSize {
        download_bytes: total,
        install_bytes: total,
    })
}
