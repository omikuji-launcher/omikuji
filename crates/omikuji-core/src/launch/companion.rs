use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use super::{EnvPurpose, build_launch_as};
use crate::library::{CompanionWhen, Game};
use crate::process::GAME_ID_VAR;
use crate::store::steam::local::with_steam_wine;
use crate::template_vars::TemplateVars;

struct Invocation {
    target: String,
    args: Vec<String>,
}

pub async fn start(game: &Game, host_env: &HashMap<String, String>) {
    let Some(companion) = resolve(game) else {
        return;
    };
    let delay = game.launch.companion_delay as u64;

    match game.launch.companion_when {
        CompanionWhen::Before => {
            spawn_logged(game, host_env, &companion);
            if delay > 0 {
                tokio::time::sleep(Duration::from_secs(delay)).await;
            }
        }
        CompanionWhen::After => {
            let game = game.clone();
            let env = host_env.clone();
            std::thread::spawn(move || {
                if delay > 0 {
                    std::thread::sleep(Duration::from_secs(delay));
                }
                spawn_logged(&game, &env, &companion);
            });
        }
        CompanionWhen::Wrap => {}
    }
}

fn spawn_logged(game: &Game, host_env: &HashMap<String, String>, companion: &Invocation) {
    match spawn(game, host_env, companion) {
        Ok(pid) => tracing::info!(pid, "running companion `{}`", companion.target),
        Err(e) => tracing::error!("failed to run companion `{}`: {}", companion.target, e),
    }
}

fn spawn(game: &Game, host_env: &HashMap<String, String>, companion: &Invocation) -> Result<u32> {
    let mut cmd = if game.launch.companion_needs_wine() {
        prefix_command(game, companion)?
    } else {
        host_command(host_env, companion)
    };
    cmd.env(GAME_ID_VAR, &game.metadata.id);
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

// host stands for anything that isnt wine. i have no clue how to name itb etter and its confusing to me too
fn host_command(env: &HashMap<String, String>, companion: &Invocation) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
        .arg(format!("{} \"$@\"", companion.target))
        .arg("sh")
        .args(&companion.args)
        .envs(env);
    cmd
}

fn prefix_command(game: &Game, companion: &Invocation) -> Result<Command> {
    let steam = with_steam_wine(game)?;
    let source = steam.as_ref().unwrap_or(game);

    let exe = PathBuf::from(&companion.target);
    let name = exe
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Program")
        .to_string();

    let mut tool = Game::new(name, exe)
        .with_prefix(super::resolve_prefix(source).to_string_lossy())
        .with_runner_version(source.wine.version.clone());
    tool.launch.args = companion.args.clone();
    tool.source = game.source.clone();
    tool.metadata.slug = game.slug();

    build_launch_as(&tool, EnvPurpose::Companion)?.to_command()
}

fn resolve(game: &Game) -> Option<Invocation> {
    let vars = TemplateVars::for_game(game);
    let target = vars.expand(&game.launch.companion);
    (!target.is_empty()).then(|| Invocation {
        args: game
            .launch
            .spliced_companion_args()
            .iter()
            .map(|a| vars.expand(a))
            .collect(),
        target,
    })
}
