// todo: remove one day lol

use crate::components_config::{self, ArchiveSource, ComponentsConfig};
use crate::settings::{self, Settings};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LegacyFile {
    runners: Vec<ArchiveSource>,
    dll_packs: Vec<ArchiveSource>,
    paths: LegacyPaths,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LegacyPaths {
    runners_dir: String,
    dll_packs_dir: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct LegacyState {
    dll_packs: HashMap<String, String>,
}

fn state_path() -> PathBuf {
    crate::data_dir().join("components_state.toml")
}

fn legacy_file() -> LegacyFile {
    fs::read_to_string(settings::settings_path())
        .ok()
        .and_then(|body| toml::from_str(&body).ok())
        .unwrap_or_default()
}

fn has_legacy_sections(legacy: &LegacyFile) -> bool {
    !legacy.runners.is_empty()
        || !legacy.dll_packs.is_empty()
        || !legacy.paths.dll_packs_dir.is_empty()
}

pub fn pending() -> bool {
    state_path().exists()
        || crate::data_dir().join("gog").is_dir()
        || has_legacy_sections(&legacy_file())
}

fn default_base() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omikuji")
}

fn custom_path(stored: &str, old_default: &Path) -> Option<PathBuf> {
    if stored.is_empty() {
        return None;
    }
    let expanded = settings::expand(stored);
    (expanded != old_default).then_some(expanded)
}

pub fn run(mut on_line: impl FnMut(String)) -> Result<()> {
    let legacy = legacy_file();
    let base = default_base();
    let runners_custom = custom_path(&legacy.paths.runners_dir, &base.join("runners"));
    let layers_custom = custom_path(&legacy.paths.dll_packs_dir, &base.join("components"));
    let components_root = crate::components_dir();

    if layers_custom.is_none() {
        nest_layers(&components_root, &mut on_line)?;
    }
    move_runners(
        runners_custom.clone().unwrap_or_else(|| base.join("runners")),
        runners_custom.clone().unwrap_or_else(|| components_root.join("runners")),
        if runners_custom.is_some() { "" } else { "components/runners/" },
        &mut on_line,
    )?;
    move_gog(&mut on_line)?;
    lift_config(&legacy, &mut on_line)?;
    rewrite_settings(runners_custom, layers_custom, &mut on_line)?;
    on_line("all done".into());
    Ok(())
}

fn nest_layers(root: &Path, on_line: &mut impl FnMut(String)) -> Result<()> {
    let Ok(entries) = fs::read_dir(root) else {
        return Ok(());
    };
    let moves: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| {
            !matches!(
                p.file_name().and_then(|n| n.to_str()),
                Some("layers") | Some("runners")
            )
        })
        .collect();
    if moves.is_empty() {
        return Ok(());
    }
    let layers = root.join("layers");
    fs::create_dir_all(&layers)?;
    for src in moves {
        let Some(name) = src.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };
        let dest = layers.join(&name);
        if dest.exists() {
            on_line(format!("skipping {} (already at destination)", name));
            continue;
        }
        fs::rename(&src, &dest).with_context(|| format!("moving {}", src.display()))?;
        on_line(format!("layers: {} -> components/layers/{}", name, name));
    }
    Ok(())
}

fn move_runners(
    old_root: PathBuf,
    new_root: PathBuf,
    dest_prefix: &str,
    on_line: &mut impl FnMut(String),
) -> Result<()> {
    let Ok(entries) = fs::read_dir(&old_root) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };
        let (dest, label) = match crate::archive_source::installed_source_tag(&path) {
            Some((source, tag)) if !source.is_empty() && !tag.is_empty() => {
                let label = format!("{}/{}", source, tag);
                (new_root.join(&source).join(&tag), label)
            }
            _ => (new_root.join(&name), name.clone()),
        };
        if dest == path {
            continue;
        }
        if dest.exists() {
            on_line(format!("skipping {} (already at destination)", name));
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&path, &dest).with_context(|| format!("moving {}", path.display()))?;
        on_line(format!("runners: {} -> {}{}", name, dest_prefix, label));
    }
    if old_root != new_root {
        let _ = fs::remove_dir(&old_root);
    }
    Ok(())
}

fn move_gog(on_line: &mut impl FnMut(String)) -> Result<()> {
    let old = crate::data_dir().join("gog");
    if !old.is_dir() {
        return Ok(());
    }
    let new = crate::gog::gog_dir();
    if new.exists() {
        on_line("skipping gog (already at destination)".into());
        return Ok(());
    }
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&old, &new).context("moving gog data")?;
    on_line("gog -> runtime/gog".into());
    Ok(())
}

fn lift_config(legacy: &LegacyFile, on_line: &mut impl FnMut(String)) -> Result<()> {
    if !components_config::config_path().exists() {
        let mut config = ComponentsConfig::default();
        if !legacy.runners.is_empty() {
            config.runners = legacy.runners.clone();
        }
        if !legacy.dll_packs.is_empty() {
            config.layers = legacy.dll_packs.clone();
        }
        if let Ok(body) = fs::read_to_string(state_path()) {
            config.active = toml::from_str::<LegacyState>(&body)
                .unwrap_or_default()
                .dll_packs;
        }
        components_config::save(&config).context("writing components.toml")?;
        on_line("sources + active layers -> components.toml".into());
    }
    if state_path().exists() {
        fs::remove_file(state_path()).context("removing components_state.toml")?;
        on_line("removed components_state.toml".into());
    }
    Ok(())
}

fn rewrite_settings(
    runners_custom: Option<PathBuf>,
    layers_custom: Option<PathBuf>,
    on_line: &mut impl FnMut(String),
) -> Result<()> {
    let body = fs::read_to_string(settings::settings_path()).context("reading settings.toml")?;
    let mut updated: Settings = toml::from_str(&body).unwrap_or_default();
    updated.paths.runners_dir = runners_custom
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    updated.paths.layers_dir = layers_custom
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    settings::save(&updated).context("rewriting settings.toml")?;
    on_line("settings.toml cleaned up".into());
    Ok(())
}
