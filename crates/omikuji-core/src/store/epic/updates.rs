use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use super::{installed_record, legendary_command, legendary_dir, legendary_output};
use crate::store::UpdateInfo;

#[derive(Deserialize)]
struct AssetEntry {
    app_name: String,
    build_version: String,
}

pub fn refresh_assets_cache() -> Result<()> {
    legendary_output(legendary_command()?.args(["list", "--third-party", "--json"]))?;
    Ok(())
}

pub fn find_update_for(app_id: &str) -> Option<UpdateInfo> {
    let installed_entry = installed_record(app_id)?;

    let assets_raw = std::fs::read_to_string(legendary_dir().join("assets.json")).ok()?;
    let assets: HashMap<String, Vec<AssetEntry>> = serde_json::from_str(&assets_raw).ok()?;
    let asset_list = assets.get(&installed_entry.platform)?;
    let asset = asset_list.iter().find(|a| a.app_name == app_id)?;

    if installed_entry.version == asset.build_version {
        return None;
    }

    Some(UpdateInfo {
        from_version: installed_entry.version,
        to_version: asset.build_version.clone(),
    })
}
