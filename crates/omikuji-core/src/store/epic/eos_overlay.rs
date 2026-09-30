use super::{legendary_command, legendary_dir, legendary_output};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn eos_overlay_dir() -> PathBuf {
    crate::runtime_dir().join("eos_overlay")
}

pub fn is_installed() -> bool {
    eos_overlay_dir()
        .join("EOSOverlayRenderer-Win64-Shipping.exe")
        .exists()
        || legendary_dir().join("overlay_install.json").exists()
}

pub fn install() -> Result<()> {
    let path = eos_overlay_dir();
    fs_err::create_dir_all(&path)?;

    tracing::info!("installing EOS overlay to {} ...", path.display());
    legendary_output(
        legendary_command()?
            .args(["eos-overlay", "install", "--path"])
            .arg(&path)
            .arg("-y"),
    )?;
    Ok(())
}

pub fn enable(prefix: &Path) -> Result<()> {
    if !is_installed() {
        install()?;
    }

    tracing::info!("enabling EOS overlay for prefix {} ...", prefix.display());
    legendary_output(
        legendary_command()?
            .args(["eos-overlay", "enable", "--prefix"])
            .arg(prefix),
    )?;
    Ok(())
}

pub fn disable(prefix: &Path) -> Result<()> {
    tracing::info!("disabling EOS overlay for prefix {} ...", prefix.display());
    legendary_output(
        legendary_command()?
            .args(["eos-overlay", "disable", "--prefix"])
            .arg(prefix),
    )?;
    Ok(())
}
