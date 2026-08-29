use crate::store::epic::source::find_legendary;
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn eos_overlay_dir() -> PathBuf {
    crate::runtime_dir().join("eos_overlay")
}

pub fn is_installed() -> bool {
    eos_overlay_dir()
        .join("EOSOverlayRenderer-Win64-Shipping.exe")
        .exists()
        || dirs::config_dir()
            .map(|c| c.join("legendary").join("overlay_install.json").exists())
            .unwrap_or(false)
}

pub fn install() -> Result<()> {
    let bin = find_legendary().ok_or_else(|| anyhow!("legendary not found"))?;
    let path = eos_overlay_dir();
    std::fs::create_dir_all(&path)?;

    tracing::info!("installing EOS overlay to {} ...", path.display());

    let status = Command::new(&bin)
        .arg("eos-overlay")
        .arg("install")
        .arg("--path")
        .arg(&path)
        .arg("-y")
        .status()?;

    if !status.success() {
        anyhow::bail!("legendary eos-overlay install failed");
    }

    Ok(())
}

pub fn enable(prefix: &Path) -> Result<()> {
    if !is_installed() {
        install()?;
    }

    let bin = find_legendary().ok_or_else(|| anyhow!("legendary not found"))?;

    tracing::info!("enabling EOS overlay for prefix {} ...", prefix.display());

    let status = Command::new(&bin)
        .arg("eos-overlay")
        .arg("enable")
        .arg("--prefix")
        .arg(prefix)
        .status()?;

    if !status.success() {
        anyhow::bail!("legendary eos-overlay enable failed");
    }

    Ok(())
}

pub fn disable(prefix: &Path) -> Result<()> {
    let bin = find_legendary().ok_or_else(|| anyhow!("legendary not found"))?;

    tracing::info!("disabling EOS overlay for prefix {} ...", prefix.display());

    let status = Command::new(&bin)
        .arg("eos-overlay")
        .arg("disable")
        .arg("--prefix")
        .arg(prefix)
        .status()?;

    if !status.success() {
        anyhow::bail!("legendary eos-overlay disable failed");
    }

    Ok(())
}
