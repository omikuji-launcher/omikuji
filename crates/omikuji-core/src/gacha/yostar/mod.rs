pub mod api;
mod auth;
pub mod source;
pub mod update;

use anyhow::{Result, anyhow, bail};

use crate::gacha::manifest::GachaManifest;

const PUBLISHER_SLUG: &str = "yostar";

#[derive(Debug, Clone)]
pub struct EditionApi {
    pub api_url: String,
    pub cdn_url: String,
    pub game_tag: String,
    pub salt: String,
}

pub fn edition_api(manifest: &GachaManifest, edition_id: &str) -> Result<EditionApi> {
    let edition = manifest
        .editions
        .iter()
        .find(|e| e.id == edition_id)
        .ok_or_else(|| {
            anyhow!(
                "edition '{}' not found in manifest '{}'",
                edition_id,
                manifest.id
            )
        })?;

    let field = |name: &str| -> Result<String> {
        cfg_str(&edition.strategy_config, name)
            .or_else(|| cfg_str(&manifest.strategy_config, name))
            .ok_or_else(|| {
                anyhow!(
                    "no strategy_config.{} in manifest {} for edition {}",
                    name,
                    manifest.id,
                    edition_id
                )
            })
    };

    Ok(EditionApi {
        api_url: field("api_url")?.trim_end_matches('/').to_string(),
        cdn_url: field("cdn_url")?.trim_end_matches('/').to_string(),
        game_tag: field("game_tag")?,
        salt: field("salt")?,
    })
}

fn cfg_str(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

fn disk_game_tag(manifest: &GachaManifest, install_path: &std::path::Path) -> Option<String> {
    for edition in &manifest.editions {
        let info = install_path.join(&edition.data_folder).join("app.info");
        if let Ok(text) = std::fs::read_to_string(&info)
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
        .find(|e| cfg_str(&e.strategy_config, "game_tag").as_deref() == Some(tag.as_str()))
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
        .editions
        .iter()
        .find(|e| e.id == edition_id)
        .and_then(|e| cfg_str(&e.strategy_config, "game_tag"))
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

pub fn installed_version(game_slug: &str, edition: &str) -> Option<String> {
    crate::gacha::state::read_installed_version(PUBLISHER_SLUG, game_slug, edition)
}

pub fn set_installed_version(game_slug: &str, edition: &str, version: &str) {
    crate::gacha::state::write_installed_version(PUBLISHER_SLUG, game_slug, edition, version);
}

// no unity fallback: package and client versions differ, a wrong stamp = phantom update
pub fn read_install_version(install_path: &std::path::Path, _data_folder: &str) -> Option<String> {
    crate::gacha::state::read_install_dotversion(install_path)
}

pub fn cleanup_yostar_state(
    _app_id: &str,
    _install_path: &std::path::Path,
    _temp_dir: Option<&std::path::Path>,
) {
}
