use anyhow::Result;
use std::path::PathBuf;

use super::{ComponentMissing, runtime_dir};
use crate::fs_util::{find_executable_in_paths, is_executable};
use crate::library::Game;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WineVariant {
    System,
    Runner,
    // proton requires umu-launcher
    Proton,
}

fn looks_like_proton(s: &str) -> bool {
    s.starts_with("GE-Proton")
        || s.starts_with("Proton")
        || s.starts_with("dwproton")
        || s.starts_with("proton")
}

impl WineVariant {
    pub fn from_version(version: &str) -> Self {
        if version.is_empty() || version == "system" {
            return WineVariant::System;
        }
        match crate::runners::runner_dir(version) {
            Some(dir) if crate::runners::is_proton_dir(&dir) => WineVariant::Proton,
            Some(_) => WineVariant::Runner,
            None if looks_like_proton(version.strip_prefix("steam:").unwrap_or(version)) => {
                WineVariant::Proton
            }
            None => WineVariant::Runner,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtonVerb {
    Run,
    WaitForExitAndRun,
    RunInPrefix,
}

impl ProtonVerb {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::WaitForExitAndRun => "waitforexitandrun",
            Self::RunInPrefix => "runinprefix",
        }
    }
}

// for proton this returns umu-run, not wine; the actual proton path is set via PROTONPATH env in build_env
pub fn resolve_wine_exe(variant: WineVariant, version: &str) -> Result<PathBuf> {
    if version.starts_with("steam:") {
        let steam_version = version.strip_prefix("steam:").unwrap_or(version);
        return resolve_steam_runner(steam_version);
    }

    if let Some(name) = version.strip_prefix("system:") {
        if let Some(path) = crate::runners::system_wine_paths().get(name) {
            return Ok(path.clone());
        }
        anyhow::bail!("Runner `{}` not found.", name);
    }

    match variant {
        WineVariant::System => Ok(PathBuf::from("wine")),
        WineVariant::Runner => crate::runners::installed_runner_dir(version)
            .map(|d| d.join("bin").join("wine"))
            .filter(|p| p.exists())
            .ok_or_else(|| anyhow::anyhow!("Runner `{}` not found.", version)),
        WineVariant::Proton => {
            let umu_run = find_umu_run().ok_or_else(|| {
                anyhow::Error::new(ComponentMissing {
                    name: "umu-run".to_string(),
                })
            })?;

            let has_files = crate::runners::installed_runner_dir(version)
                .map(|d| d.join("files").exists())
                .unwrap_or(false);
            if !has_files {
                anyhow::bail!("Runner `{}` not found.", version);
            }

            Ok(umu_run)
        }
    }
}

pub fn missing_component(game: &Game) -> Option<String> {
    if matches!(
        game.runner.runner_type.as_str(),
        "steam" | "flatpak" | "native"
    ) {
        return None;
    }
    let version = &game.wine.version;
    resolve_wine_exe(WineVariant::from_version(version), version)
        .err()?
        .downcast_ref::<ComponentMissing>()
        .map(|c| c.name.clone())
}

fn resolve_steam_runner(version: &str) -> Result<PathBuf> {
    if crate::runners::steam_runners_ignored() {
        anyhow::bail!("Runner `{}` not found.", version);
    }
    crate::store::steam::local::find_proton_install(version)
        .ok_or_else(|| anyhow::anyhow!("Runner `{}` not found.", version))?;
    find_umu_run().ok_or_else(|| {
        anyhow::Error::new(ComponentMissing {
            name: "umu-run".to_string(),
        })
    })
}

pub fn umu_system_path() -> Option<PathBuf> {
    const SYSTEM_PATHS: &[&str] = &[
        "/app/share/umu/umu-run",
        "/usr/share/umu/umu-run",
        "/usr/local/share/umu/umu-run",
        "/opt/umu/umu-run",
    ];
    find_executable_in_paths(&["umu-run", "umu_run.py"], SYSTEM_PATHS)
}

pub fn find_umu_run() -> Option<PathBuf> {
    if let Some(p) = umu_system_path() {
        return Some(p);
    }
    let our_runtime = runtime_dir().join("umu-run");
    (our_runtime.exists() && is_executable(&our_runtime)).then_some(our_runtime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wine_variant_from_version() {
        assert_eq!(WineVariant::from_version(""), WineVariant::System);
        assert_eq!(WineVariant::from_version("system"), WineVariant::System);
        assert_eq!(
            WineVariant::from_version("wine-ge-9-5"),
            WineVariant::Runner
        );
        assert_eq!(WineVariant::from_version("lutris-7.2"), WineVariant::Runner);
        assert_eq!(
            WineVariant::from_version("GE-Proton10-34"),
            WineVariant::Proton
        );
        assert_eq!(
            WineVariant::from_version("Proton-9-0-4"),
            WineVariant::Proton
        );
    }
}
