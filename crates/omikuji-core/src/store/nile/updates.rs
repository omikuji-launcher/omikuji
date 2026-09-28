use std::sync::{LazyLock, Mutex};

use crate::store::UpdateInfo;

static PENDING: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn refresh_updates_cache() -> Option<()> {
    let output = super::blocking_command()
        .ok()?
        .arg("list-updates")
        .arg("--json")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let ids: Vec<String> = serde_json::from_slice(&output.stdout).ok()?;
    *PENDING.lock().ok()? = ids;
    Some(())
}

pub fn find_update_for(app_id: &str) -> Option<UpdateInfo> {
    if !PENDING.lock().ok()?.iter().any(|id| id == app_id) {
        return None;
    }
    let installed = super::read_installed().remove(app_id)?;
    Some(UpdateInfo {
        from_version: installed.version,
        to_version: String::new(),
    })
}
