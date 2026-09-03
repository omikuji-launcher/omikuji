use std::collections::HashMap;
use std::path::{Path, PathBuf};

// legendary's installed.json shape, we copy it for gog cause yes
pub struct Entry {
    pub install_path: PathBuf,
    pub executable: String,
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
    let mut map = HashMap::new();
    let Ok(content) = std::fs::read_to_string(path) else {
        return map;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else {
        tracing::warn!("installed registry parse failed: {}", path.display());
        return map;
    };
    let Some(obj) = v.as_object() else {
        return map;
    };
    for (app_name, data) in obj {
        let Some(install_path) = data.get("install_path").and_then(|p| p.as_str()) else {
            continue;
        };
        map.insert(
            app_name.clone(),
            Entry {
                install_path: PathBuf::from(install_path),
                executable: data
                    .get("executable")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string(),
                title: data.get("title").and_then(|t| t.as_str()).map(String::from),
            },
        );
    }
    map
}

pub fn entry(path: &Path, app_name: &str) -> Option<Entry> {
    read(path).remove(app_name)
}
