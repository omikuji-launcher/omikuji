use anyhow::Result;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::{ProtonVerb, WineVariant, build_launch};
use crate::library::{AlongsideWhen, Game};
use crate::template_vars::TemplateVars;

pub async fn start(game: &Game, host_env: &HashMap<String, String>) {
    let Some(target) = companion_of(game) else {
        return;
    };
    let delay = game.launch.alongside_delay as u64;

    match game.launch.alongside_when {
        AlongsideWhen::Before => {
            spawn_logged(game, host_env, &target);
            if delay > 0 {
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }
        }
        AlongsideWhen::After => {
            let game = game.clone();
            let env = host_env.clone();
            std::thread::spawn(move || {
                if delay > 0 {
                    std::thread::sleep(Duration::from_secs(delay));
                }
                spawn_logged(&game, &env, &target);
            });
        }
    }
}

fn spawn_logged(game: &Game, host_env: &HashMap<String, String>, target: &str) {
    match spawn(game, host_env, target) {
        Ok(pid) => tracing::info!(pid, "running `{}` alongside the game", target),
        Err(e) => tracing::error!("failed to run `{}` alongside the game: {}", target, e),
    }
}

fn spawn(game: &Game, host_env: &HashMap<String, String>, target: &str) -> Result<u32> {
    let mut cmd = match game.runner.runner_type.as_str() {
        "native" | "flatpak" => host_command(game, host_env, target),
        _ => prefix_command(game, target)?,
    };
    cmd.env(crate::process::GAME_ID_VAR, &game.metadata.id);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    #[cfg(unix)]
    unsafe {
        use std::os::unix::process::CommandExt;
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let mut child = cmd.spawn()?;
    let pid = child.id();
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(pid)
}

fn host_command(game: &Game, env: &HashMap<String, String>, target: &str) -> Command {
    let mut line = target.to_string();
    if !game.launch.alongside_args.is_empty() {
        line.push(' ');
        line.push_str(&game.launch.alongside_args.join(" "));
    }
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(line).envs(env);
    cmd
}

fn prefix_command(game: &Game, target: &str) -> Result<Command> {
    let steam = crate::store::steam::local::with_steam_wine(game)?;
    let source = steam.as_ref().unwrap_or(game);

    let exe = std::path::PathBuf::from(target);
    let name = exe
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Program")
        .to_string();

    let mut tool = Game::with_options(
        name,
        exe,
        (!source.wine.prefix.is_empty()).then(|| source.wine.prefix.clone()),
        Some("wine".to_string()),
        (!source.wine.version.is_empty()).then(|| source.wine.version.clone()),
    );
    tool.launch.args = game.launch.alongside_args.clone();

    let mut cmd = build_launch(&tool)?.to_command()?;
    if WineVariant::from_version(&tool.wine.version) == WineVariant::Proton {
        cmd.env("PROTON_VERB", ProtonVerb::RunInPrefix.as_str());
    }
    Ok(cmd)
}

fn companion_of(game: &Game) -> Option<String> {
    let target = TemplateVars::for_game(game).expand(&game.launch.alongside);
    (!target.is_empty()).then_some(target)
}
