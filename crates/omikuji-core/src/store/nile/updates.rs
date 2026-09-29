use std::sync::{LazyLock, Mutex, PoisonError};

use anyhow::{Result, bail};

use crate::store::UpdateInfo;

static PENDING: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn refresh_updates_cache() -> Result<()> {
    let output = super::blocking_command()?
        .args(["list-updates", "--json"])
        .output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("nile list-updates failed: {}", err.trim());
    }
    let ids: Vec<String> = serde_json::from_slice(&output.stdout)?;
    *PENDING.lock().unwrap_or_else(PoisonError::into_inner) = ids;
    Ok(())
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
