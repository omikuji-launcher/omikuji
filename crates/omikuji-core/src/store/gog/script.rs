use std::collections::HashMap;
use std::iter;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Result;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::launch::{ProtonVerb, WineVariant, resolve_prefix, wine_command};
use crate::library::Game;
use crate::prefixes::host_path_to_wine;

const INTERPRETER_ID: &str = "ISI";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    #[serde(default)]
    build_id: String,
    #[serde(default)]
    script_interpreter: bool,
}

fn manifest(app_id: &str) -> Option<Manifest> {
    serde_json::from_value(super::gogdl_manifest(app_id)?).ok()
}

pub fn needs_interpreter(app_id: &str) -> bool {
    manifest(app_id).is_some_and(|m| m.script_interpreter)
}

fn redist_dir() -> PathBuf {
    super::gog_dir().join("redist")
}

fn interpreter_path() -> PathBuf {
    redist_dir()
        .join("__redist")
        .join(INTERPRETER_ID)
        .join("scriptinterpreter.exe")
}

pub fn fetch_interpreter() -> Result<()> {
    let output = super::gogdl_command()?
        .arg("redist")
        .arg("--ids")
        .arg(INTERPRETER_ID)
        .arg("--path")
        .arg(redist_dir())
        .stdin(Stdio::null())
        .output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gogdl redist failed: {}", err.trim());
    }
    Ok(())
}

pub fn run_install_scripts(
    game: &Game,
    wine_exe: &Path,
    env: &HashMap<String, String>,
) -> Result<()> {
    let app_id = game.effective_app_id();
    let Some(manifest) = manifest(app_id).filter(|m| m.script_interpreter) else {
        return Ok(());
    };
    let Some(info) = super::find_installed_info(app_id) else {
        return Ok(());
    };
    let prefix = resolve_prefix(game);

    let pending: Vec<(String, String)> = iter::once(app_id.to_string())
        .chain(super::installed_dlcs(app_id).into_iter().map(|d| d.id))
        .map(|id| (script_digest(&info.install_path, &id), id))
        .filter(|(digest, id)| !marker_matches(&prefix, id, digest))
        .collect();
    if pending.is_empty() {
        return Ok(());
    }

    if !interpreter_path().exists() {
        fetch_interpreter()?;
    }
    let run = ScriptRun {
        wine_exe,
        env,
        variant: WineVariant::from_version(&game.wine.version),
        install_path: &info.install_path,
        support_dir: super::support_dir(app_id),
        build_id: &manifest.build_id,
    };
    for (digest, id) in pending {
        let status = run.command(&id).status()?;
        if !status.success() {
            anyhow::bail!("scriptinterpreter for {id} exited with {status}");
        }
        fs_err::write(marker_path(&prefix, &id), digest)?;
        tracing::info!("gog install script applied for {id}");
    }
    Ok(())
}

struct ScriptRun<'a> {
    wine_exe: &'a Path,
    env: &'a HashMap<String, String>,
    variant: WineVariant,
    install_path: &'a Path,
    support_dir: PathBuf,
    build_id: &'a str,
}

impl ScriptRun<'_> {
    fn command(&self, product_id: &str) -> Command {
        let mut cmd = wine_command(
            self.wine_exe,
            self.env,
            self.variant,
            Some(ProtonVerb::WaitForExitAndRun),
            [
                interpreter_path().to_string_lossy().into_owned(),
                "/VERYSILENT".to_string(),
                format!("/DIR={}", host_path_to_wine(self.install_path)),
                format!("/Language={}", super::LANG_NAME),
                format!("/LANG={}", super::LANG_NAME),
                format!("/ProductId={product_id}"),
                "/galaxyclient".to_string(),
                format!("/buildId={}", self.build_id),
                format!("/lang-code={}", super::LANG_CODE),
                format!("/supportDir={}", host_path_to_wine(&self.support_dir)),
                // gog's own typo
                "/nodesktopshorctut".to_string(),
                "/nodesktopshortcut".to_string(),
            ],
        );
        cmd.current_dir(self.install_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    }
}

fn script_digest(install_path: &Path, product_id: &str) -> String {
    let script =
        fs_err::read(install_path.join(format!("goggame-{product_id}.script"))).unwrap_or_default();
    format!("{:x}", Sha256::digest(script))
}

fn marker_path(prefix: &Path, product_id: &str) -> PathBuf {
    prefix.join(format!(".omikuji-gog-script-{product_id}"))
}

fn marker_matches(prefix: &Path, product_id: &str, digest: &str) -> bool {
    fs_err::read_to_string(marker_path(prefix, product_id)).is_ok_and(|m| m == digest)
}
