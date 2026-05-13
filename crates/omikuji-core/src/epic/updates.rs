use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

pub struct EpicUpdateInfo {
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
}

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

pub fn blocking_check_epic_update(app_id: &str) -> Option<EpicUpdateInfo> {
    let _ = refresh_assets_cache();
    find_update_for(app_id)
}

pub fn refresh_assets_cache() -> Option<()> {
    let bin = crate::downloads::legendary::find_legendary()?;
    let _ = Command::new(bin)
        .args(["list", "--third-party", "--json"])
        .output()
        .ok()?;
    Some(())
}

pub fn find_update_for(app_id: &str) -> Option<EpicUpdateInfo> {
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

    Some(EpicUpdateInfo {
        from_version: installed_entry.version.clone(),
        to_version: asset.build_version.clone(),
        download_size: 0,
    })
}

fn legendary_config_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("legendary"))
}
