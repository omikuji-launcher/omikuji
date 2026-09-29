use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::store::UpdateInfo;
use crate::store::epic::source::require_legendary;

#[derive(Deserialize)]
struct InstalledMeta {
    version: String,
    platform: String,
}

#[derive(Deserialize)]
struct AssetEntry {
    app_name: String,
    build_version: String,
}

pub fn refresh_assets_cache() -> Result<()> {
    let output = Command::new(require_legendary()?)
        .args(["list", "--third-party", "--json"])
        .output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("legendary list failed: {}", err.trim());
    }
    Ok(())
}

pub fn find_update_for(app_id: &str) -> Option<UpdateInfo> {
    let config = legendary_config_dir()?;

    let installed_raw = std::fs::read_to_string(config.join("installed.json")).ok()?;
    let installed: HashMap<String, InstalledMeta> = serde_json::from_str(&installed_raw).ok()?;
    let installed_entry = installed.get(app_id)?;

    let assets_raw = std::fs::read_to_string(config.join("assets.json")).ok()?;
    let assets: HashMap<String, Vec<AssetEntry>> = serde_json::from_str(&assets_raw).ok()?;
    let asset_list = assets.get(&installed_entry.platform)?;
    let asset = asset_list.iter().find(|a| a.app_name == app_id)?;

    if installed_entry.version == asset.build_version {
        return None;
    }

    Some(UpdateInfo {
        from_version: installed_entry.version.clone(),
        to_version: asset.build_version.clone(),
    })
}

fn legendary_config_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("legendary"))
}
