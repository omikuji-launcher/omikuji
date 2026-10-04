use crate::fs_util::write_executable_atomic;
use crate::library::{Game, Library, generate_id, id_from_slug_id, rfc3339_now};
use crate::log_files;
use crate::media::{self, MediaType, media_path};
use crate::settings;
use crate::store::steam;
use anyhow::{Context, Result};
use fs_err as fs;
use nix::sys::statvfs::statvfs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn desktop_dir() -> PathBuf {
    settings::expand(&settings::get().paths.desktop_dir)
}

pub fn applications_dir() -> PathBuf {
    if std::env::var("FLATPAK_ID").is_ok() {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/applications");
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("applications")
}

pub fn icons_dir() -> PathBuf {
    if std::env::var("FLATPAK_ID").is_ok() {
        return dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/icons/hicolor/256x256/apps");
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("icons/hicolor/256x256/apps")
}

// im doing this just for a stupid icon on a stupid dock. hmph.
pub fn ensure_steam_icon(game: &Game) -> Result<()> {
    let src = media_path(&game.metadata.id, &MediaType::Icon);
    if !src.exists() {
        return Ok(());
    }

    let dir = icons_dir();
    fs::create_dir_all(&dir)?;

    let appid = steam::synthetic_appid(&game.metadata.id);
    let link = dir.join(format!("steam_icon_{}.png", appid));
    let _ = fs::remove_file(&link);
    fs_err::os::unix::fs::symlink(&src, &link)?;

    Ok(())
}

pub fn remove_steam_icon(game_id: &str) {
    let appid = steam::synthetic_appid(game_id);
    let _ = fs::remove_file(icons_dir().join(format!("steam_icon_{}.png", appid)));
}

pub fn browse_files(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    let url = if path_str.starts_with("file://") {
        path_str.to_string()
    } else {
        format!("file://{}", path_str)
    };

    Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .with_context(|| format!("failed to open file manager for {}", path.display()))?;

    Ok(())
}

pub fn get_game_browse_dir(game: &Game) -> Option<PathBuf> {
    if !game.launch.working_dir.is_empty() {
        let path = PathBuf::from(&game.launch.working_dir);
        if path.exists() {
            return Some(path);
        }
    }

    if game.runner.runner_type.is_steam() {
        return steam::local::get_game_install_dir(&game.metadata.id);
    }

    game.metadata.exe.parent().map(|p| p.to_path_buf())
}

fn desktop_filename(slug: &str, id: &str) -> String {
    format!("omikuji.{}-{}.desktop", slug, id)
}

fn launcher_command() -> String {
    if let Ok(app_id) = std::env::var("FLATPAK_ID") {
        format!("flatpak run {}", app_id)
    } else {
        "omikuji".to_string()
    }
}

fn generate_desktop_content(game: &Game) -> String {
    let icon = resolve_desktop_icon(game);
    let exec = format!(
        "{} run {} --notify-gui",
        launcher_command(),
        launch_target(game)
    );

    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Icon={}\n\
         Exec={}\n\
         Categories=Game\n\
         Terminal=false\n",
        escape_desktop_value(&game.metadata.name),
        icon,
        exec
    )
}

fn resolve_desktop_icon(game: &Game) -> String {
    if !game.metadata.icon.is_empty() {
        return game.metadata.icon.clone();
    }

    let icon = media_path(&game.metadata.id, &MediaType::Icon);
    if icon.exists() {
        return icon.to_string_lossy().into_owned();
    }

    let coverart = media_path(&game.metadata.id, &MediaType::Coverart);
    if coverart.exists() {
        return coverart.to_string_lossy().into_owned();
    }

    "omikuji".to_string()
}

fn escape_desktop_value(value: &str) -> String {
    value.replace("\\", "\\\\").replace("\n", "\\n")
}

pub fn game_slug(game: &Game) -> String {
    game.slug()
}

pub fn launch_target(game: &Game) -> String {
    game.metadata.id.clone()
}

fn shortcut_path(game: &Game, dir: &Path) -> PathBuf {
    dir.join(desktop_filename(&game_slug(game), &game.metadata.id))
}

fn exec_launches(entry: &str, id: &str) -> bool {
    entry
        .lines()
        .filter_map(|line| line.strip_prefix("Exec="))
        .flat_map(str::split_whitespace)
        .any(|arg| id_from_slug_id(arg) == id)
}

fn shortcuts_in(game: &Game, dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "desktop"))
        .filter(|p| {
            fs::read_to_string(p).is_ok_and(|entry| exec_launches(&entry, &game.metadata.id))
        })
        .collect()
}

fn write_shortcut(game: &Game, dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let path = shortcuts_in(game, dir)
        .into_iter()
        .next()
        .unwrap_or_else(|| shortcut_path(game, dir));
    write_executable_atomic(&path, generate_desktop_content(game))?;
    Ok(path)
}

fn remove_shortcut(game: &Game, dir: &Path) -> Result<()> {
    for path in shortcuts_in(game, dir) {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn create_desktop_shortcut(game: &Game) -> Result<PathBuf> {
    write_shortcut(game, &desktop_dir())
}

pub fn create_menu_shortcut(game: &Game) -> Result<PathBuf> {
    write_shortcut(game, &applications_dir())
}

pub fn remove_desktop_shortcut(game: &Game) -> Result<()> {
    remove_shortcut(game, &desktop_dir())
}

pub fn remove_menu_shortcut(game: &Game) -> Result<()> {
    remove_shortcut(game, &applications_dir())
}

pub fn desktop_shortcut_exists(game: &Game) -> bool {
    !shortcuts_in(game, &desktop_dir()).is_empty()
}

pub fn menu_shortcut_exists(game: &Game) -> bool {
    !shortcuts_in(game, &applications_dir()).is_empty()
}

pub fn duplicate_game(game: &Game) -> Result<Game> {
    let new_id = generate_id();

    let mut new_game = game.clone();
    new_game.metadata.id = new_id;
    new_game.metadata.name = format!("{} (Copy)", game.metadata.name);
    new_game.metadata.playtime = 0.0;
    new_game.metadata.last_played = String::new();
    new_game.metadata.added = rfc3339_now();

    Library::save_game(&new_game)?;

    Ok(new_game)
}

pub fn delete_game(game: &Game) -> Result<()> {
    let id = &game.metadata.id;
    Library::remove_game_file(id)?;
    media::remove_cached_media(id);
    if let Some(logs) = log_files::existing_game_logs_dir(id)
        && let Err(e) = fs::remove_dir_all(&logs)
    {
        tracing::warn!("delete_game: logs for {id}: {e}");
    }
    let shortcuts = [
        ("desktop", remove_desktop_shortcut(game)),
        ("menu", remove_menu_shortcut(game)),
        ("steam", steam::shortcuts::remove_shortcut(game)),
    ];
    for (kind, result) in shortcuts {
        if let Err(e) = result {
            tracing::warn!("delete_game: {kind} shortcut for {id}: {e}");
        }
    }
    Ok(())
}

pub fn disk_free_space(path: &str) -> u64 {
    let mut p = std::path::Path::new(path).to_path_buf();
    while !p.exists() {
        if !p.pop() {
            return 0;
        }
    }
    match statvfs(&p) {
        Ok(stat) => stat.fragment_size() * stat.blocks_available(),
        Err(e) => {
            tracing::error!("statvfs failed for {}: {}", p.display(), e);
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_desktop_filename() {
        assert_eq!(
            desktop_filename("elden-ring", "abc123"),
            "omikuji.elden-ring-abc123.desktop"
        );
    }

    #[test]
    fn test_get_game_browse_dir() {
        let game = Game::new("Test".to_string(), PathBuf::from("/games/test/game.exe"));
        let dir = get_game_browse_dir(&game).unwrap();
        assert_eq!(dir, PathBuf::from("/games/test"));
    }

    #[test]
    fn test_get_game_browse_dir_with_working_dir() {
        use std::env;

        let temp_dir = env::temp_dir();
        let mut game = Game::new("Test".to_string(), PathBuf::from("/games/test/game.exe"));
        game.launch.working_dir = temp_dir.to_string_lossy().to_string();
        let dir = get_game_browse_dir(&game).unwrap();
        assert_eq!(dir, temp_dir);
    }

    #[test]
    fn test_duplicate_game_resets_fields() {
        let mut game = Game::new("Test".to_string(), PathBuf::from("/games/test/game.exe"));
        game.metadata.id = "original".to_string();
        game.metadata.playtime = 123.5;
        game.metadata.last_played = "Apr 15, 2026".to_string();

        let dup = duplicate_game(&game).unwrap();

        assert_ne!(dup.metadata.id, "original");
        assert_eq!(dup.metadata.name, "Test (Copy)");
        assert_eq!(dup.metadata.playtime, 0.0);
        assert_eq!(dup.metadata.last_played, "");
        assert_eq!(dup.metadata.exe, game.metadata.exe);
    }
}
