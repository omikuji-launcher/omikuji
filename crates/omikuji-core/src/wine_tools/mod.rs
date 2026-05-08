use crate::launch::{build_env, resolve_wine_exe, WineVariant};
use crate::library::Game;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[derive(Debug, Clone)]
pub enum WineTool {
    Winecfg,
    Winetricks,
    Regedit,
    Cmd,
    Winefile,
    RunExe(PathBuf),
    // wineserver -k (or wineboot -k for proton). useful when a crashed game leaves wineserver running (took it from lutris).
    KillWineserver,
}

pub fn run(game: &Game, tool: WineTool) -> Result<Child> {
    // steam-imported games need the prefix rewritten to compatdata/{app_id}/pfx; 
    // we must not create our own omikuji prefix dir becuase steam owns the lifetime of that one.
    // also rewrite wine.version from "steam:{app_id}" to "steam:{proton_dir_name}" using the version stamp in compatdata/version so PROTONPATH resolves correctly.
    let mut effective: Game;
    let g: &Game = if game.source.kind == "steam" && !game.source.app_id.is_empty() {
        let pfx = crate::steam::local::find_steam_prefix(&game.source.app_id)
            .ok_or_else(|| anyhow!(
                "no Steam prefix for this game yet — launch it through Steam \
                 at least once so Steam creates compatdata/{}/pfx",
                game.source.app_id
            ))?;

        let stamped = crate::steam::local::find_steam_proton_version(&game.source.app_id);
        let install = crate::steam::local::resolve_or_default_proton(stamped.as_deref())
            .ok_or_else(|| anyhow!(
                "no Proton install found — install one via Steam or drop \
                 a GE-Proton build in ~/.local/share/Steam/compatibilitytools.d/"
            ))?;
        let dir_name = install
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Proton".to_string());

        effective = game.clone();
        effective.wine.version = format!("steam:{}", dir_name);
        effective.wine.prefix = pfx.to_string_lossy().into_owned();
        &effective
    } else {
        game
    };

    let variant = WineVariant::from_version(&g.wine.version);
    let wine_exe = resolve_wine_exe(variant, &g.wine.version)?;
    let env = build_env(g, variant, &wine_exe);

    let (program, args) = build_command(&tool, variant, &wine_exe)?;

    let mut cmd = Command::new(&program);
    cmd.args(&args);
    // replace rather than extend so WINEPREFIX etc. from build_env win over anything inherited from the launcher's env
    cmd.env_clear();
    cmd.envs(&env);

    // proton tools (except the kill verb) need waitforexitandrun so umu-run waits for the tool to close before obliterating the prefix down
    if variant == WineVariant::Proton && !matches!(tool, WineTool::KillWineserver) {
        cmd.env("PROTON_VERB", "waitforexitandrun");
    }

    eprintln!(
        "[wine_tools] {:?} :: {} {}",
        tool,
        program.display(),
        args.join(" ")
    );
    cmd.spawn().map_err(|e| anyhow!("failed to spawn wine tool: {}", e))
}

fn build_command(
    tool: &WineTool,
    variant: WineVariant,
    wine_exe: &Path,
) -> Result<(PathBuf, Vec<String>)> {
    match tool {
        WineTool::Winecfg => Ok((wine_exe.to_path_buf(), vec!["winecfg".into()])),
        WineTool::Regedit => Ok((wine_exe.to_path_buf(), vec!["regedit".into()])),
        WineTool::Cmd => Ok((wine_exe.to_path_buf(), vec!["wineconsole".into()])),
        WineTool::Winefile => Ok((wine_exe.to_path_buf(), vec!["winefile".into()])),
        WineTool::RunExe(path) => Ok((
            wine_exe.to_path_buf(),
            vec![path.to_string_lossy().into_owned()],
        )),
        WineTool::Winetricks => {
            if variant == WineVariant::Proton {
                // umu-run ships its own winetricks verb apparently
                Ok((
                    wine_exe.to_path_buf(),
                    vec!["winetricks".into(), "--gui".into()],
                ))
            } else {
                let wt = find_winetricks()?;
                Ok((wt, vec!["--gui".into()]))
            }
        }
        WineTool::KillWineserver => {
            if variant == WineVariant::Proton {
                // wineboot -k tears down the session cleanly; invoking wineserver directly races with umu's lifecycle
                Ok((
                    wine_exe.to_path_buf(),
                    vec!["wineboot".into(), "-k".into()],
                ))
            } else {
                let parent = wine_exe.parent().unwrap_or(Path::new("."));
                let ws = parent.join("wineserver");
                let bin = if ws.exists() { ws } else { PathBuf::from("wineserver") };
                Ok((bin, vec!["-k".into()]))
            }
        }
    }
}

fn find_winetricks() -> Result<PathBuf> {
    if let Ok(output) = Command::new("which").arg("winetricks").output()
        && output.status.success() {
            let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !p.is_empty() {
                return Ok(PathBuf::from(p));
            }
        }
    let bundled = crate::runtime_dir().join("winetricks");
    if bundled.exists() {
        return Ok(bundled);
    }
    anyhow::bail!(
        "winetricks not found. install via your distro (package 'winetricks') \
         or drop the script at ~/.local/share/omikuji/runtime/winetricks"
    )
}
