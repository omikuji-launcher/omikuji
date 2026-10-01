pub mod api;
mod auth;
pub mod source;
pub mod update;

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::gacha::manifest::{GachaManifest, ManifestEdition};
use crate::gacha::state;

#[derive(Debug, Clone, Deserialize)]
pub struct EditionApi {
    pub api_url: String,
    pub cdn_url: String,
    pub game_tag: String,
    pub salt: String,
}

pub fn edition_api(manifest: &GachaManifest, edition_id: &str) -> Result<EditionApi> {
    let mut api: EditionApi = manifest.strategy_config(manifest.require_edition(edition_id)?)?;
    for url in [&mut api.api_url, &mut api.cdn_url] {
        url.truncate(url.trim_end_matches('/').len());
    }
    Ok(api)
}

fn game_tag(manifest: &GachaManifest, edition: &ManifestEdition) -> Option<String> {
    manifest
        .strategy_config::<EditionApi>(edition)
        .ok()
        .map(|api| api.game_tag)
}

fn disk_game_tag(manifest: &GachaManifest, install_path: &std::path::Path) -> Option<String> {
    for edition in &manifest.editions {
        let info = install_path.join(&edition.data_folder).join("app.info");
        if let Ok(text) = fs_err::read_to_string(&info)
            && let Some(tag) = text.lines().nth(1).map(str::trim).filter(|s| !s.is_empty())
        {
            return Some(tag.to_string());
        }
    }
    None
}

pub fn detect_edition(manifest: &GachaManifest, install_path: &std::path::Path) -> Option<String> {
    let tag = disk_game_tag(manifest, install_path)?;
    manifest
        .editions
        .iter()
        .find(|e| game_tag(manifest, e).as_deref() == Some(tag.as_str()))
        .map(|e| e.id.clone())
}

pub fn verify_edition_on_disk(
    manifest: &GachaManifest,
    edition_id: &str,
    install_path: &std::path::Path,
) -> Result<()> {
    let Some(found) = disk_game_tag(manifest, install_path) else {
        return Ok(());
    };
    let expected = manifest
        .edition(edition_id)
        .and_then(|e| game_tag(manifest, e))
        .unwrap_or_default();
    if !expected.is_empty() && found != expected {
        bail!(
            "{} holds {}, not {}",
            install_path.display(),
            found,
            expected
        );
    }
    Ok(())
}

// no unity fallback: package and client versions differ, a wrong stamp = phantom update
pub fn read_install_version(install_path: &std::path::Path, _data_folder: &str) -> Option<String> {
    state::read_install_dotversion(install_path)
}
