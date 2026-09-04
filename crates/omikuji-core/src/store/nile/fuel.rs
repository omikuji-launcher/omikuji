use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Fuel {
    pub main: Main,
    #[serde(default)]
    pub post_install: Vec<PostInstall>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Main {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_subdir_override: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PostInstall {
    pub command: String,
}

fn from_windows_path(raw: &str) -> PathBuf {
    PathBuf::from(raw.replace('\\', "/"))
}

impl Fuel {
    pub fn exe(&self, install_root: &Path) -> PathBuf {
        install_root.join(from_windows_path(&self.main.command))
    }

    pub fn working_dir_override(&self, install_root: &Path) -> Option<PathBuf> {
        let sub = self.main.working_subdir_override.as_deref()?;
        (!sub.is_empty()).then(|| install_root.join(from_windows_path(sub)))
    }
}

pub fn path_in(install_root: &Path) -> PathBuf {
    install_root.join("fuel.json")
}

pub fn load(install_root: &Path) -> Result<Fuel> {
    let path = path_in(install_root);
    let raw =
        std::fs::read_to_string(&path).map_err(|e| anyhow!("read {}: {}", path.display(), e))?;
    json5::from_str(&raw).map_err(|e| anyhow!("parse {}: {}", path.display(), e))
}
