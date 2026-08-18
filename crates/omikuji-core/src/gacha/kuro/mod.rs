pub mod api;
pub mod krpdiff;
mod patcher;
pub mod source;
pub mod update;

use crate::gacha::manifest::GachaManifest;
use anyhow::{Result, anyhow};

pub fn index_url_from_manifest(manifest: &GachaManifest, edition_id: &str) -> Result<String> {
    manifest
        .editions
        .iter()
        .find(|e| e.id == edition_id)
        .and_then(|e| e.strategy_config.get("index_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            anyhow!(
                "no strategy_config.index_url in manifest {} for edition {}",
                manifest.id,
                edition_id
            )
        })
}

pub fn parse_app_id(app_id: &str) -> Result<(String, String)> {
    let mut parts = app_id.splitn(2, ':');
    let game = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("invalid kuro app_id: {}", app_id))?
        .to_string();
    let edition = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("invalid kuro app_id: {}", app_id))?
        .to_string();
    Ok((game, edition))
}

const PUBLISHER_SLUG: &str = "kurogame";

pub fn installed_version(game_slug: &str, edition: &str) -> Option<String> {
    crate::gacha::state::read_installed_version(PUBLISHER_SLUG, game_slug, edition)
}

pub fn set_installed_version(game_slug: &str, edition: &str, version: &str) {
    crate::gacha::state::write_installed_version(PUBLISHER_SLUG, game_slug, edition, version);
}

pub fn read_install_version(install_path: &std::path::Path, data_folder: &str) -> Option<String> {
    use crate::gacha::state;
    if let Some(v) = state::read_install_dotversion(install_path) {
        return Some(v);
    }
    if let Some(v) = read_package_version_json(install_path) {
        return Some(v);
    }
    if let Some(v) = read_wuwa_resources_version(install_path) {
        return Some(v);
    }
    if let Some(v) = state::scan_globalgamemanagers(install_path, data_folder, b'_') {
        return Some(v);
    }
    state::scan_globalgamemanagers(install_path, data_folder, 0)
}

fn read_wuwa_resources_version(install_path: &std::path::Path) -> Option<String> {
    let resources_dir = install_path.join("Client/Saved/Resources");
    let entries = std::fs::read_dir(&resources_dir).ok()?;

    let mut best: Option<(u32, u32, u32)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let parts: Vec<&str> = name_str.split('.').collect();
        if parts.len() != 3 {
            continue;
        }
        let Ok(a) = parts[0].parse::<u32>() else {
            continue;
        };
        let Ok(b) = parts[1].parse::<u32>() else {
            continue;
        };
        let Ok(c) = parts[2].parse::<u32>() else {
            continue;
        };
        let v = (a, b, c);
        if best.is_none_or(|bv| v > bv) {
            best = Some(v);
        }
    }

    best.map(|(a, b, c)| format!("{}.{}.{}", a, b, c))
}

fn read_package_version_json(install_path: &std::path::Path) -> Option<String> {
    let s = std::fs::read_to_string(install_path.join("version.json")).ok()?;
    let colon = s.find(':')?;
    let after = &s[colon + 1..];
    let end = after.find('}').unwrap_or(after.len());
    let v = after[..end].trim().trim_matches('"');
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

pub fn cleanup_kuro_state(
    _app_id: &str,
    _install_path: &std::path::Path,
    _temp_dir: Option<&std::path::Path>,
) {
}
