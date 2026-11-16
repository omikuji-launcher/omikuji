use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::{GogStore, gog_auth_path, gog_dir, read_credentials};
use crate::components::{self, SettingsKey};
use crate::launch::{ProtonVerb, WineVariant, resolve_prefix, wine_command};
use crate::library::Game;

const SERVICE_DIR: &str = "drive_c/ProgramData/GOG.com/Galaxy/redists";
const SERVICE_EXE: &str = "GalaxyCommunication.exe";
const SERVICE_WINE_PATH: &str = r"C:\ProgramData\GOG.com\Galaxy\redists\GalaxyCommunication.exe";

pub struct Comet(Child);

impl Comet {
    pub fn child(&mut self) -> &mut Child {
        &mut self.0
    }
}

impl Drop for Comet {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub async fn start(game: &Game, wine_exe: &Path, env: &HashMap<String, String>) -> Option<Comet> {
    if !game.is_gog() || !game.uses_wine_prefix() || !game.source.comet {
        return None;
    }
    try_start(game, wine_exe, env)
        .await
        .map(Comet)
        .inspect_err(|e| tracing::warn!("comet: {e:#} (launching without it)"))
        .ok()
}

async fn try_start(game: &Game, wine_exe: &Path, env: &HashMap<String, String>) -> Result<Child> {
    components::ensure(&components::comet_tools()).await?;
    install_service(game, wine_exe, env)?;

    read_credentials().await?;
    let comet = components::path_for(SettingsKey::Comet).context("comet is not installed")?;
    let mut cmd = Command::new(comet);
    cmd.env("XDG_CONFIG_PATH", heroic_config_root()?)
        .arg("--from-heroic")
        .arg("--username")
        .arg(GogStore::new().display_name)
        .arg("--quit")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    Ok(cmd.spawn()?)
}

fn heroic_config_root() -> Result<PathBuf> {
    let root = gog_dir().join("comet");
    let dir = root.join("heroic/gog_store");
    let link = dir.join("auth.json");
    if fs_err::symlink_metadata(&link).is_err() {
        fs_err::create_dir_all(&dir)?;
        fs_err::os::unix::fs::symlink(gog_auth_path(), &link)?;
    }
    Ok(root)
}

fn install_service(game: &Game, wine_exe: &Path, env: &HashMap<String, String>) -> Result<()> {
    let dir = resolve_prefix(game).join(SERVICE_DIR);
    let dest = dir.join(SERVICE_EXE);
    if dest.exists() {
        return Ok(());
    }
    let src = components::path_for(SettingsKey::GalaxyDummy)
        .context("GalaxyCommunication.exe is not installed")?;

    let variant = WineVariant::from_version(&game.wine.version);
    let binpath = format!("binpath={SERVICE_WINE_PATH}");
    let commands: [&[&str]; 2] = [
        &["sc", "create", "GalaxyCommunication", &binpath],
        &[
            "reg",
            "add",
            r"HKLM\SOFTWARE\WOW6432Node\GOG.com\GalaxyClient\paths",
            "/v",
            "client",
            "/t",
            "REG_SZ",
            "/d",
            r"C:\Program Files\GOG Galaxy",
            "/f",
        ],
    ];
    for args in commands {
        let status = wine_command(
            wine_exe,
            env,
            variant,
            Some(ProtonVerb::WaitForExitAndRun),
            args,
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
        if !status.success() {
            bail!("`{}` exited with {status}", args.join(" "));
        }
    }

    fs_err::create_dir_all(&dir)?;
    fs_err::copy(src, &dest)?;
    Ok(())
}
