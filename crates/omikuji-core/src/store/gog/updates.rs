use std::path::Path;

use serde::Deserialize;

use crate::store::UpdateInfo;

#[derive(Deserialize)]
struct GoggameInfo {
    #[serde(rename = "buildId")]
    build_id: Option<String>,
    #[serde(rename = "versionName")]
    version_name: Option<String>,
}

pub fn blocking_check_gog_update(app_id: &str) -> Option<UpdateInfo> {
    let installed_info = super::find_installed_info(app_id)?;
    let (installed_build, installed_version) =
        read_installed_meta(&installed_info.install_path, app_id)?;
    let latest_build = fetch_latest_build(app_id)?;
    if installed_build == latest_build {
        return None;
    }
    Some(UpdateInfo {
        from_version: installed_version.unwrap_or(installed_build),
        to_version: latest_build,
    })
}

fn read_installed_meta(install_path: &Path, app_id: &str) -> Option<(String, Option<String>)> {
    let info_path = install_path.join(format!("goggame-{}.info", app_id));
    let content = fs_err::read_to_string(info_path).ok()?;
    let info: GoggameInfo = serde_json::from_str(&content).ok()?;
    let build_id = info.build_id?;
    Some((build_id, info.version_name))
}

fn fetch_latest_build(app_id: &str) -> Option<String> {
    let output = super::gogdl_command()
        .ok()?
        .args(["info", app_id, "--os", "windows"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    super::latest_build_id(&serde_json::from_slice(&output.stdout).ok()?)
}
