use std::path::Path;
use std::process::Command;

use serde::Deserialize;

pub struct GogUpdateInfo {
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
}

#[derive(Deserialize)]
struct GoggameInfo {
    #[serde(rename = "buildId")]
    build_id: Option<String>,
    #[serde(rename = "versionName")]
    version_name: Option<String>,
}

pub fn blocking_check_gog_update(app_id: &str) -> Option<GogUpdateInfo> {
    let installed_info = crate::gog::find_installed_info(app_id)?;
    let (installed_build, installed_version) = read_installed_meta(&installed_info.install_path, app_id)?;
    let latest_build = fetch_latest_build(app_id)?;
    if installed_build == latest_build {
        return None;
    }
    Some(GogUpdateInfo {
        from_version: installed_version.unwrap_or(installed_build),
        to_version: latest_build,
        download_size: 0,
    })
}

fn read_installed_meta(install_path: &Path, app_id: &str) -> Option<(String, Option<String>)> {
    let info_path = install_path.join(format!("goggame-{}.info", app_id));
    let content = std::fs::read_to_string(info_path).ok()?;
    let info: GoggameInfo = serde_json::from_str(&content).ok()?;
    let build_id = info.build_id?;
    Some((build_id, info.version_name))
}

fn fetch_latest_build(app_id: &str) -> Option<String> {
    let bin = super::gogdl_bin().ok()?;
    let auth = super::gog_auth_path();
    let gogdl_cfg = super::gogdl_config_dir();
    let _ = std::fs::create_dir_all(&gogdl_cfg);
    let output = Command::new(&bin)
        .env("GOGDL_CONFIG_PATH", &gogdl_cfg)
        .arg("--auth-config-path")
        .arg(&auth)
        .arg("info")
        .arg(app_id)
        .arg("--os")
        .arg("windows")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    v.pointer("/builds/items/0/build_id")
        .and_then(|b| b.as_str())
        .map(String::from)
}
