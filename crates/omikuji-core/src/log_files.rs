use crate::app_settings::AppSettings;
use crate::library::{Game, id_from_slug_id};
use std::cmp::Reverse;
use std::path::{Path, PathBuf};

pub fn app_logs_dir() -> PathBuf {
    crate::logs_dir().join("app")
}

fn games_logs_dir() -> PathBuf {
    crate::logs_dir().join("games")
}

// the folder keeps the slug it was created with, so a renamed game still finds its history by id
pub fn existing_game_logs_dir(game_id: &str) -> Option<PathBuf> {
    fs_err::read_dir(games_logs_dir())
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| id_from_slug_id(name) == game_id)
        })
}

pub fn game_logs_dir(game: &Game) -> PathBuf {
    existing_game_logs_dir(game.id()).unwrap_or_else(|| games_logs_dir().join(game.slug_with_id()))
}

pub fn next_log_path(dir: &Path, stem: &str) -> std::io::Result<PathBuf> {
    fs_err::create_dir_all(dir)?;
    let keep = AppSettings::load().behavior.logs_kept.clamp(1, 10) as usize;
    prune(dir, keep - 1);
    Ok(dir.join(format!(
        "{stem}_{}.log",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    )))
}

fn prune(dir: &Path, keep: usize) {
    let Ok(entries) = fs_err::read_dir(dir) else {
        return;
    };
    let mut logs: Vec<_> = entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "log"))
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .collect();
    logs.sort_by_key(|(modified, _)| Reverse(*modified));
    for (_, path) in logs.into_iter().skip(keep) {
        if let Err(e) = fs_err::remove_file(&path) {
            tracing::warn!("{e}");
        }
    }
}
