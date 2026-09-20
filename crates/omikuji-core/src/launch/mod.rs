use anyhow::Result;
use std::path::PathBuf;

use crate::fs_util::is_executable;
use crate::library::Game;
use crate::template_vars::TemplateVars;

pub mod alongside;
mod assemble;
mod command;
mod env;
mod prefix;
mod wine;

pub use assemble::ResolvedLaunch;
pub use command::wine_command;
pub use env::{EnvPurpose, build_env};
pub use prefix::{
    effective_prefix, prefix_path_for, prepare_epic_prefix, resolve_prefix, wineserver_alive,
};
pub use wine::{
    ProtonVerb, WineVariant, find_umu_run, missing_component, resolve_wine_exe, umu_system_path,
};

use assemble::assemble_launch;

#[derive(Debug)]
pub struct ComponentMissing {
    pub name: String,
}

impl std::fmt::Display for ComponentMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` not found", self.name)
    }
}

impl std::error::Error for ComponentMissing {}

#[derive(Debug)]
pub struct StoreSignedOut {
    pub store: String,
}

impl std::fmt::Display for StoreSignedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "You are signed out of {}, so it cannot start the game.",
            self.store
        )
    }
}

impl std::error::Error for StoreSignedOut {}

pub fn prepare_launch(game: &Game) -> Result<ResolvedLaunch> {
    let config = assemble_launch(game, EnvPurpose::Session)?;
    reject_slop_env(&config)?;
    run_pre_launch_script(game, &config);
    validate_exe(game)?;
    Ok(config)
}

pub fn build_launch(game: &Game) -> Result<ResolvedLaunch> {
    build_launch_as(game, EnvPurpose::Session)
}

pub fn build_launch_as(game: &Game, purpose: EnvPurpose) -> Result<ResolvedLaunch> {
    let config = assemble_launch(game, purpose)?;
    reject_slop_env(&config)?;
    validate_exe(game)?;
    Ok(config)
}

fn reject_slop_env(config: &ResolvedLaunch) -> Result<()> {
    if config.env.contains_key("WINE_CANONICAL_HOLE") {
        anyhow::bail!(
            "WINE_CANONICAL_HOLE detected in the launch environment. bro remove this shit pls. this variable is not real, wine has no canonical hole, and whatever slop config it came from probably broke other things too :xdd:"
        );
    }
    Ok(())
}

fn run_pre_launch_script(game: &Game, config: &ResolvedLaunch) {
    let script = &TemplateVars::for_game(game).expand(&game.launch.pre_launch_script);
    if script.is_empty() {
        return;
    }
    tracing::info!("running pre-launch script: {}", script);
    let cwd = if config.working_dir.exists() {
        config.working_dir.clone()
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    };
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(script.as_str())
        .current_dir(&cwd)
        .envs(&config.env)
        .status();
    match status {
        Ok(s) if !s.success() => tracing::warn!("pre-launch script exited with: {}", s),
        Err(e) => tracing::error!("pre-launch script failed: {}", e),
        _ => {}
    }
}

fn validate_exe(game: &Game) -> Result<()> {
    let exe = &game.metadata.exe;
    match game.runner.runner_type.as_str() {
        "steam" | "flatpak" => Ok(()),
        "native" => {
            if exe.as_os_str().is_empty() {
                anyhow::bail!("Native runner requires an executable");
            }
            if !exe.exists() {
                anyhow::bail!("Game executable not found at `{}`", exe.display());
            }
            if !is_executable(exe) {
                anyhow::bail!(
                    "`{}` is not executable. Mark it executable (chmod +x) and try again.",
                    exe.display()
                );
            }
            Ok(())
        }
        _ => {
            if !game.is_epic() && !exe.as_os_str().is_empty() && !exe.exists() {
                anyhow::bail!("Game executable not found at `{}`", exe.display());
            }
            Ok(())
        }
    }
}
