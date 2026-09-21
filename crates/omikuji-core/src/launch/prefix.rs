use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use super::wine::{ProtonVerb, WineVariant};
use super::wine_command;
use crate::library::{Game, RunnerType};
use crate::template_vars::TemplateVars;

pub fn prefix_path_for(game: &Game) -> PathBuf {
    if !game.wine.prefix.is_empty() {
        return PathBuf::from(TemplateVars::base(game).expand(&game.wine.prefix));
    }

    let dir = crate::prefixes_dir();

    // layout: prefixes/{slug}-{id}. if the name slugifies to nothing (e.g. non-ascii title) fall back to just the id so the dir is unique.
    let slug = game.slug();
    let folder = if slug.is_empty() {
        game.metadata.id.clone()
    } else {
        format!("{}-{}", slug, game.metadata.id)
    };
    dir.join(folder)
}

pub fn effective_prefix(game: &Game) -> Option<PathBuf> {
    match game.runner.runner_type {
        RunnerType::Native | RunnerType::Flatpak => None,
        RunnerType::Steam => {
            if game.source.app_id.is_empty() {
                None
            } else {
                crate::store::steam::local::find_steam_prefix(&game.source.app_id)
            }
        }
        RunnerType::Wine => Some(prefix_path_for(game)),
    }
}

fn server_lock_paths(prefix: &Path) -> Vec<PathBuf> {
    use std::os::unix::fs::MetadataExt;

    let base = std::env::temp_dir().join(format!(".wine-{}", unsafe { libc::getuid() }));
    [prefix.to_path_buf(), prefix.join("pfx")]
        .iter()
        .filter_map(|dir| dir.metadata().ok())
        .map(|meta| {
            base.join(format!("server-{:x}-{:x}", meta.dev(), meta.ino()))
                .join("lock")
        })
        .collect()
}

pub fn wineserver_alive(prefix: &Path) -> bool {
    use std::os::unix::io::AsRawFd;

    server_lock_paths(prefix).iter().any(|lock| {
        let Ok(file) = std::fs::File::open(lock) else {
            return false;
        };
        let taken = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0;
        if taken {
            unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) };
        }
        !taken
    })
}

pub fn resolve_prefix(game: &Game) -> PathBuf {
    let prefix = prefix_path_for(game);
    if game.wine.prefix.is_empty()
        && !prefix.exists()
        && let Err(e) = std::fs::create_dir_all(&prefix)
    {
        tracing::error!("failed to create prefix dir: {}", e);
    }
    prefix
}

pub fn prepare_epic_prefix(
    game: &Game,
    wine_exe: &Path,
    env: &HashMap<String, String>,
) -> Result<()> {
    let prefix = resolve_prefix(game);

    // spoof the epic launcher registry key so games that check for it dont bail early
    let mut cmd = wine_command(
        wine_exe,
        env,
        WineVariant::from_version(&game.wine.version),
        Some(ProtonVerb::WaitForExitAndRun),
        [
            "reg",
            "add",
            "HKEY_CLASSES_ROOT\\com.epicgames.launcher",
            "/f",
        ],
    );
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    if let Err(e) = cmd.status() {
        tracing::error!("epic registry spoof failed: {}", e);
    }

    let dummy_src = crate::runtime_dir().join("EpicGamesLauncher.exe");
    if dummy_src.exists() {
        let dest_dir = crate::prefixes::windows_dir(&prefix).join("command");
        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            tracing::error!("failed to create command dir in prefix: {}", e);
        } else {
            let dest_file = dest_dir.join("EpicGamesLauncher.exe");
            if !dest_file.exists()
                && let Err(e) = std::fs::copy(&dummy_src, &dest_file)
            {
                tracing::error!("failed to copy dummy EpicGamesLauncher.exe: {}", e);
            }
        }
    }

    Ok(())
}
