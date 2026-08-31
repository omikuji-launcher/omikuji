use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::library::{AlongsideWhen, Game};
use crate::template_vars::TemplateVars;

const EPIC_SCRIPT_NAME: &str = "omikuji-alongside.bat";

pub fn wine_launcher(game: &Game, exe: &Path, args: &[String]) -> Result<Option<PathBuf>> {
    let (Some(companion), false) = (companion_of(game), exe.as_os_str().is_empty()) else {
        return Ok(None);
    };

    let game_line = start_line(&exe.to_string_lossy(), args, None);

    let path = script_path(game);
    write_script(&path, game, game_line, &companion)?;
    Ok(Some(path))
}

pub fn epic_launcher(game: &Game, install_path: &Path, exe: &Path) -> Result<Option<String>> {
    if !install_path.is_dir() {
        return Ok(None);
    }

    let (Some(companion), false) = (companion_of(game), exe.as_os_str().is_empty()) else {
        let _ = std::fs::remove_file(install_path.join(EPIC_SCRIPT_NAME));
        return Ok(None);
    };

    let working_dir = exe.parent().unwrap_or(install_path).to_string_lossy();
    let mut game_line = start_line(&exe.to_string_lossy(), &[], Some(&working_dir));
    game_line.push_str(" %*");

    write_script(
        &install_path.join(EPIC_SCRIPT_NAME),
        game,
        game_line,
        &companion,
    )?;
    Ok(Some(EPIC_SCRIPT_NAME.to_string()))
}

fn write_script(path: &Path, game: &Game, game_line: String, companion: &str) -> Result<()> {
    let companion_line = start_line(companion, &game.launch.alongside_args, None);
    let wait = wait_line(game.launch.alongside_delay);

    let ordered = match game.launch.alongside_when {
        AlongsideWhen::Before => [companion_line, wait, game_line],
        AlongsideWhen::After => [game_line, wait, companion_line],
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    write!(file, "@echo off\r\n")?;
    for line in ordered.iter().filter(|l| !l.is_empty()) {
        write!(file, "{line}\r\n")?;
    }
    Ok(())
}

pub fn start_host(game: &Game, env: &HashMap<String, String>) {
    let Some(mut target) = companion_of(game) else {
        return;
    };
    if !game.launch.alongside_args.is_empty() {
        target.push(' ');
        target.push_str(&game.launch.alongside_args.join(" "));
    }

    let delay = match game.launch.alongside_when {
        AlongsideWhen::Before => 0,
        AlongsideWhen::After => game.launch.alongside_delay,
    };

    let game_id = game.metadata.id.clone();
    let env = env.clone();
    std::thread::spawn(move || {
        if delay > 0 {
            std::thread::sleep(Duration::from_secs(delay as u64));
        }
        match spawn_detached(&target, &game_id, &env) {
            Ok(pid) => tracing::info!(pid, "running `{}` alongside the game", target),
            Err(e) => tracing::error!("failed to run `{}` alongside the game: {}", target, e),
        }
    });
}

fn companion_of(game: &Game) -> Option<String> {
    let target = TemplateVars::for_game(game).expand(&game.launch.alongside);
    (!target.is_empty()).then_some(target)
}

fn script_path(game: &Game) -> PathBuf {
    crate::cache_dir()
        .join("alongside")
        .join(format!("{}.bat", game.metadata.id))
}

fn start_line(unix_path: &str, args: &[String], working_dir: Option<&str>) -> String {
    let mut line = match working_dir {
        Some(dir) => format!(
            "start \"\" /b /d \"z:{}\" \"z:{}\"",
            escape(dir),
            escape(unix_path)
        ),
        None => format!("start \"\" /b \"z:{}\"", escape(unix_path)),
    };
    if !args.is_empty() {
        line.push(' ');
        line.push_str(&escape(&args.join(" ")));
    }
    line
}

fn wait_line(delay: u32) -> String {
    if delay == 0 {
        return String::new();
    }
    format!("ping -n {} 127.0.0.1 >nul", delay + 1)
}

fn escape(s: &str) -> String {
    s.replace('%', "%%")
}

fn spawn_detached(target: &str, game_id: &str, env: &HashMap<String, String>) -> Result<u32> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(target).envs(env);
    cmd.env(crate::process::GAME_ID_VAR, game_id);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let mut child = cmd.spawn()?;
    let pid = child.id();
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(pid)
}
