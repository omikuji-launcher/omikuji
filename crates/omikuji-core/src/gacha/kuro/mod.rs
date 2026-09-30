pub mod api;
pub mod krpdiff;
mod patcher;
pub mod source;
pub mod update;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use indexmap::IndexMap;
use serde::Deserialize;

use crate::gacha::file_sync::sanitize_rel;
use crate::gacha::manifest::{GachaManifest, LaunchEffect};

#[derive(Debug, Clone, Deserialize)]
pub struct KuroConfig {
    pub index_url: String,
    #[serde(default)]
    pub default_pack: Option<String>,
    #[serde(default)]
    pub packs: IndexMap<String, PackDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackDef {
    pub label: String,
    pub folder: String,
    #[serde(flatten)]
    pub effect: LaunchEffect,
}

impl KuroConfig {
    pub fn load(manifest: &GachaManifest, edition_id: &str) -> Result<Self> {
        let edition = manifest.require_edition(edition_id)?;
        let config: Self = serde_json::from_value(edition.strategy_config.clone())
            .with_context(|| format!("strategy_config of {} {}", manifest.id, edition_id))?;
        if let Some((id, _)) = config
            .packs
            .iter()
            .find(|(_, def)| sanitize_rel(&def.folder).as_os_str().is_empty())
        {
            bail!("pack {id} of {} has no folder", manifest.id);
        }
        Ok(config)
    }

    pub fn pack(&self, id: &str) -> Result<&PackDef> {
        self.packs
            .get(id)
            .ok_or_else(|| anyhow!("unknown pack: {id}"))
    }

    pub fn install_packs<'a>(&'a self, picked: &'a [String], game: &str) -> Result<Vec<&'a str>> {
        if !picked.is_empty() || self.packs.is_empty() {
            return Ok(picked.iter().map(String::as_str).collect());
        }
        let id = self.default_pack.as_deref().ok_or_else(|| {
            anyhow!("no pack selected for {game} and its manifest has no default_pack")
        })?;
        let def = self.pack(id).context("default_pack")?;
        tracing::warn!(
            "no texture pack selected for {game}, installing default ({})",
            def.label
        );
        Ok(vec![id])
    }

    pub fn installed_packs<'a>(&'a self, root: &'a Path) -> impl Iterator<Item = &'a str> {
        self.packs
            .iter()
            .filter(|(_, def)| def.is_installed(root))
            .map(|(id, _)| id.as_str())
    }

    pub fn remove_pack(&self, root: &Path, id: &str) -> Result<()> {
        let dir = self.pack(id)?.dir(root);
        if dir.exists() {
            fs_err::remove_dir_all(dir)?;
        }
        Ok(())
    }
}

impl PackDef {
    fn dir(&self, root: &Path) -> PathBuf {
        root.join(sanitize_rel(&self.folder))
    }

    pub fn is_installed(&self, root: &Path) -> bool {
        self.dir(root).is_dir()
    }
}

pub fn parse_app_id(app_id: &str) -> Result<(String, String)> {
    let mut parts = app_id.splitn(2, ':');
    let game = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("invalid kuro app_id: {}", app_id))?
        .to_string();
    let edition = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("invalid kuro app_id: {}", app_id))?
        .to_string();
    Ok((game, edition))
}

pub fn read_install_version(install_path: &std::path::Path, data_folder: &str) -> Option<String> {
    use crate::gacha::state;
    if let Some(v) = state::read_install_dotversion(install_path) {
        return Some(v);
    }
    if let Some(v) = read_package_version_json(install_path) {
        return Some(v);
    }
    if let Some(v) = read_wuwa_resources_version(install_path) {
        return Some(v);
    }
    if let Some(v) = state::scan_globalgamemanagers(install_path, data_folder, b'_') {
        return Some(v);
    }
    state::scan_globalgamemanagers(install_path, data_folder, 0)
}

fn read_wuwa_resources_version(install_path: &std::path::Path) -> Option<String> {
    let resources_dir = install_path.join("Client/Saved/Resources");
    let entries = fs_err::read_dir(&resources_dir).ok()?;

    let mut best: Option<(u32, u32, u32)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let parts: Vec<&str> = name_str.split('.').collect();
        if parts.len() != 3 {
            continue;
        }
        let Ok(a) = parts[0].parse::<u32>() else {
            continue;
        };
        let Ok(b) = parts[1].parse::<u32>() else {
            continue;
        };
        let Ok(c) = parts[2].parse::<u32>() else {
            continue;
        };
        let v = (a, b, c);
        if best.is_none_or(|bv| v > bv) {
            best = Some(v);
        }
    }

    best.map(|(a, b, c)| format!("{}.{}.{}", a, b, c))
}

fn read_package_version_json(install_path: &std::path::Path) -> Option<String> {
    let s = fs_err::read_to_string(install_path.join("version.json")).ok()?;
    let colon = s.find(':')?;
    let after = &s[colon + 1..];
    let end = after.find('}').unwrap_or(after.len());
    let v = after[..end].trim().trim_matches('"');
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}
