use crate::launch::{build_env, resolve_wine_exe, EnvPurpose, WineVariant};
use crate::library::Game;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub enum WineTool {
    Winecfg,
    Winetricks,
    WinetricksVerbs(Vec<String>),
    Regedit,
    Cmd,
    Explorer,
    RunExe(PathBuf),
    // wineserver -k (or wineboot -k for proton). useful when a crashed game leaves wineserver running (took it from lutris).
    KillWineserver,
    Custom(Vec<String>),
}

impl WineTool {
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "winecfg" => Self::Winecfg,
            "winetricks" => Self::Winetricks,
            "regedit" => Self::Regedit,
            "cmd" => Self::Cmd,
            "explorer" => Self::Explorer,
            "kill" | "killwineserver" => Self::KillWineserver,
            _ => return None,
        })
    }

    pub fn from_command_line(line: &str) -> Option<Self> {
        let tokens: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        (!tokens.is_empty()).then_some(Self::Custom(tokens))
    }

    fn is_winetricks(&self) -> bool {
        match self {
            Self::Winetricks | Self::WinetricksVerbs(_) => true,
            Self::Custom(args) => args.first().is_some_and(|a| a == "winetricks"),
            _ => false,
        }
    }
}

pub fn run(game: &Game, tool: WineTool) -> Result<Child> {
    let mut cmd = build_wine_command(game, &tool)?;
    cmd.spawn().map_err(|e| anyhow!("failed to spawn wine tool: {}", e))
}

pub fn run_streamed<F: FnMut(&str)>(game: &Game, tool: WineTool, mut on_line: F) -> Result<()> {
    let mut cmd = build_wine_command(game, &tool)?;
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| anyhow!("failed to spawn wine tool: {}", e))?;
    let (tx, rx) = std::sync::mpsc::channel();
    if let Some(out) = child.stdout.take() {
        pipe_lines(out, tx.clone());
    }
    if let Some(err) = child.stderr.take() {
        pipe_lines(err, tx.clone());
    }
    drop(tx);
    for line in rx {
        on_line(&line);
    }
    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("{:?} exited with {}", tool, status);
    }
    Ok(())
}

pub fn run_detached<L, D>(game: Game, tool: WineTool, on_line: L, on_done: D)
where
    L: FnMut(&str) + Send + 'static,
    D: FnOnce(bool, String) + Send + 'static,
{
    std::thread::spawn(move || {
        let (ok, err) = match run_streamed(&game, tool, on_line) {
            Ok(_) => (true, String::new()),
            Err(e) => (false, e.to_string()),
        };
        on_done(ok, err);
    });
}

fn pipe_lines<R: std::io::Read + Send + 'static>(reader: R, tx: std::sync::mpsc::Sender<String>) {
    std::thread::spawn(move || {
        use std::io::BufRead;
        for line in std::io::BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = tx.send(line);
        }
    });
}

fn build_wine_command(game: &Game, tool: &WineTool) -> Result<Command> {
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
    let mut env = build_env(g, variant, &wine_exe, EnvPurpose::Tool);

    // fuck this shit lmao
    if tool.is_winetricks()
        && let Some(bundle) = staged_ca_bundle()
    {
        let b = bundle.to_string_lossy().into_owned();
        env.insert("CURL_CA_BUNDLE".to_string(), b.clone());
        env.insert("SSL_CERT_FILE".to_string(), b);
    }

    let (program, args) = build_command(tool, variant, &wine_exe)?;

    let mut cmd = Command::new(&program);
    cmd.args(&args);
    // replace rather than extend so WINEPREFIX etc. from build_env win over anything inherited from the launcher's env
    cmd.env_clear();
    cmd.envs(&env);

    // proton tools (except the kill verb) need waitforexitandrun so umu-run waits for the tool to close before obliterating the prefix down
    if variant == WineVariant::Proton && !matches!(tool, WineTool::KillWineserver) {
        cmd.env("PROTON_VERB", "waitforexitandrun");
    }

    tracing::debug!(
        "{:?} :: {} {}",
        tool,
        program.display(),
        args.join(" ")
    );
    Ok(cmd)
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
        WineTool::Explorer => Ok((wine_exe.to_path_buf(), vec!["explorer".into()])),
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
        WineTool::WinetricksVerbs(verbs) => {
            let mut args = vec!["-q".to_string()];
            args.extend(verbs.iter().cloned());
            if variant == WineVariant::Proton {
                let mut a = vec!["winetricks".to_string()];
                a.extend(args);
                Ok((wine_exe.to_path_buf(), a))
            } else {
                Ok((find_winetricks()?, args))
            }
        }
        WineTool::Custom(args) => match args.split_first() {
            Some((first, rest)) if first == "winetricks" => {
                if variant == WineVariant::Proton {
                    Ok((wine_exe.to_path_buf(), args.clone()))
                } else {
                    Ok((find_winetricks()?, rest.to_vec()))
                }
            }
            _ => Ok((wine_exe.to_path_buf(), args.clone())),
        },
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

fn ca_bundle() -> Option<PathBuf> {
    for candidate in [
        "/etc/ssl/certs/ca-certificates.crt",
        "/etc/pki/tls/certs/ca-bundle.crt",
        "/etc/ssl/cert.pem",
        "/etc/ssl/ca-bundle.pem",
    ] {
        let path = Path::new(candidate);
        if path.exists() {
            return Some(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()));
        }
    }
    None
}

fn staged_ca_bundle() -> Option<PathBuf> {
    let src = ca_bundle()?;
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache"));
    let dir = cache.join("omikuji");
    if std::fs::create_dir_all(&dir).is_err() {
        return Some(src);
    }
    let dst = dir.join("ca-bundle.crt");
    match std::fs::copy(&src, &dst) {
        Ok(_) => Some(dst),
        Err(_) => Some(src),
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
         or drop the script at {}",
        crate::runtime_dir().join("winetricks").display()
    )
}
