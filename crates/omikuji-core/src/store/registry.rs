use crate::fs_util::write_atomic;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// legendary's installed.json shape, we copy it for gog cause yes
#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub install_path: PathBuf,
    #[serde(default)]
    pub executable: String,
    #[serde(default)]
    pub title: Option<String>,
}

impl Entry {
    pub fn has_executable(&self) -> bool {
        !self.executable.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct InstalledInfo {
    pub install_path: PathBuf,
    pub executable: PathBuf,
    pub title: Option<String>,
}

impl Entry {
    pub fn resolved(self, exe_rel: Option<String>) -> InstalledInfo {
        let executable = match exe_rel {
            Some(p) if !p.is_empty() => self.install_path.join(p),
            _ => PathBuf::new(),
        };
        InstalledInfo {
            install_path: self.install_path,
            executable,
            title: self.title,
        }
    }
}

pub fn read(path: &Path) -> HashMap<String, Entry> {
    read_as(path)
}

pub fn read_as<T: DeserializeOwned>(path: &Path) -> HashMap<String, T> {
    let Ok(content) = fs_err::read_to_string(path) else {
        return HashMap::new();
    };
    let Ok(raw) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&content) else {
        tracing::warn!("installed registry parse failed: {}", path.display());
        return HashMap::new();
    };
    raw.into_iter()
        .filter_map(|(app_name, data)| Some((app_name, serde_json::from_value(data).ok()?)))
        .collect()
}

pub fn entry(path: &Path, app_name: &str) -> Option<Entry> {
    read(path).remove(app_name)
}

pub fn insert(path: &Path, app_name: &str, entry: Entry) -> Result<()> {
    let mut entries = read(path);
    entries.insert(app_name.to_string(), entry);
    write(path, &entries)
}

pub fn remove(path: &Path, app_name: &str) -> Result<()> {
    let mut entries = read(path);
    if entries.remove(app_name).is_none() {
        return Ok(());
    }
    write(path, &entries)
}

fn write(path: &Path, entries: &HashMap<String, Entry>) -> Result<()> {
    Ok(write_atomic(path, serde_json::to_string_pretty(entries)?)?)
}
