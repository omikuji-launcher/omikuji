use std::sync::{LazyLock, Mutex};

pub struct NileUpdateInfo {
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
}

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

pub fn find_update_for(app_id: &str) -> Option<NileUpdateInfo> {
    if !PENDING.lock().ok()?.iter().any(|id| id == app_id) {
        return None;
    }
    let installed = super::read_installed().remove(app_id)?;
    Some(NileUpdateInfo {
        from_version: installed.version,
        to_version: String::new(),
        download_size: 0,
    })
}

pub fn blocking_check_nile_update(app_id: &str) -> Option<NileUpdateInfo> {
    refresh_updates_cache()?;
    find_update_for(app_id)
}
