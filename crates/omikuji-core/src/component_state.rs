// separate from settings.toml (static knobs) and ui.toml (visual prefs);
// empty string or missing key both mean disabled

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ComponentState {
    pub dll_packs: HashMap<String, String>,
}

pub fn state_path() -> PathBuf {
    crate::data_dir().join("components_state.toml")
}

static CACHE: Mutex<Option<ComponentState>> = Mutex::new(None);

pub fn get() -> ComponentState {
    let mut guard = CACHE.lock().unwrap();
    if let Some(s) = guard.as_ref() {
        return s.clone();
    }
    let s = load_from_disk();
    *guard = Some(s.clone());
    s
}

fn load_from_disk() -> ComponentState {
    let path = state_path();
    if !path.exists() {
        return ComponentState::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(body) => toml::from_str::<ComponentState>(&body).unwrap_or_else(|e| {
            tracing::warn!("couldn't parse {}: {} - using defaults", path.display(), e);
            ComponentState::default()
        }),
        Err(e) => {
            tracing::warn!("couldn't read {}: {} - using defaults", path.display(), e);
            ComponentState::default()
        }
    }
}

pub fn active_version(source_name: &str) -> String {
    get()
        .dll_packs
        .get(source_name)
        .cloned()
        .unwrap_or_default()
}

pub fn set_active_version(source_name: &str, tag: &str) -> std::io::Result<()> {
    let mut guard = CACHE.lock().unwrap();
    let mut state = guard.clone().unwrap_or_else(load_from_disk);
    if tag.is_empty() {
        state.dll_packs.remove(source_name);
    } else {
        state.dll_packs.insert(source_name.to_string(), tag.to_string());
    }
    save_inner(&state)?;
    *guard = Some(state);
    Ok(())
}

fn save_inner(state: &ComponentState) -> std::io::Result<()> {
    let body = toml::to_string_pretty(state)
        .map_err(std::io::Error::other)?;
    crate::fs_util::write_atomic(&state_path(), body)
}
