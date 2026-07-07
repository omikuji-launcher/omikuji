use crate::library::{Game, Library};
use crate::media::{media_path, MediaType};
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn desktop_dir() -> Option<PathBuf> {
    if let Ok(desktop) = std::env::var("XDG_DESKTOP_DIR") {
        let path = PathBuf::from(&desktop);
        if path.exists() {
            return Some(path);
        }
    }

    let user_dirs = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("user-dirs.dirs");


    if let Ok(content) = std::fs::read_to_string(&user_dirs) {
        for line in content.lines() {
            if line.starts_with("XDG_DESKTOP_DIR=") {
                let path_str = line.trim_start_matches("XDG_DESKTOP_DIR=")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');


                let expanded = if path_str.starts_with("$HOME/") {
                    dirs::home_dir()
                        .map(|h| h.join(&path_str[6..]))
                        .unwrap_or_else(|| PathBuf::from(path_str))
                } else if path_str.starts_with('~') {
                    dirs::home_dir()
                        .map(|h| h.join(&path_str[2..]))
                        .unwrap_or_else(|| PathBuf::from(path_str))
                } else {
                    PathBuf::from(path_str)
                };


                if expanded.exists() {
                    return Some(expanded);
                }
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let desktop = home.join("Desktop");
        if desktop.exists() {
            return Some(desktop);
        }
        // some locales use ~/desktop (lowercase)
        let desktop_lc = home.join("desktop");
        if desktop_lc.exists() {
            return Some(desktop_lc);
        }
    }

    dirs::desktop_dir()
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
    fs::create_dir_all(&dir)
        .with_context(|| format!("creating icon dir {}", dir.display()))?;

    let appid = crate::steam::synthetic_appid(&game.metadata.id);
    let link = dir.join(format!("steam_icon_{}.png", appid));
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(&src, &link)
        .with_context(|| format!("linking {}", link.display()))?;

    Ok(())
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

    if game.runner.runner_type == "steam" {
        return crate::steam::local::get_game_install_dir(&game.metadata.id);
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
    let exec = format!("{} run {}", launcher_command(), launch_target(game));

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

fn sanitize_slug(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "-")
        .replace(' ', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

pub fn game_slug(game: &Game) -> String {
    if game.metadata.slug.is_empty() {
        sanitize_slug(&game.metadata.name)
    } else {
        game.metadata.slug.clone()
    }
}

pub fn launch_target(game: &Game) -> String {
    format!("{}_{}", game_slug(game), game.metadata.id)
}

fn shortcut_path(game: &Game, dir: &Path) -> PathBuf {
    dir.join(desktop_filename(&game_slug(game), &game.metadata.id))
}

fn write_shortcut(game: &Game, path: &Path, label: &str) -> Result<()> {
    fs::write(path, generate_desktop_content(game))
        .with_context(|| format!("writing {} file {}", label, path.display()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .with_context(|| format!("setting permissions on {}", path.display()))
}

fn remove_shortcut(path: &Path, label: &str) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("removing {} file {}", label, path.display()))?;
    }
    Ok(())
}

pub fn create_desktop_shortcut(game: &Game) -> Result<PathBuf> {
    let desktop = desktop_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Desktop")))
        .context("could not find or create desktop directory")?;
    fs::create_dir_all(&desktop)
        .with_context(|| format!("creating desktop directory {}", desktop.display()))?;

    let path = shortcut_path(game, &desktop);
    write_shortcut(game, &path, "desktop")?;
    Ok(path)
}

pub fn create_menu_shortcut(game: &Game) -> Result<PathBuf> {
    let apps_dir = applications_dir();
    fs::create_dir_all(&apps_dir)
        .with_context(|| format!("creating applications directory {}", apps_dir.display()))?;

    let path = shortcut_path(game, &apps_dir);
    write_shortcut(game, &path, "menu")?;
    Ok(path)
}

pub fn remove_desktop_shortcut(game: &Game) -> Result<()> {
    match desktop_dir() {
        Some(desktop) => remove_shortcut(&shortcut_path(game, &desktop), "desktop"),
        None => Ok(()),
    }
}

pub fn remove_menu_shortcut(game: &Game) -> Result<()> {
    remove_shortcut(&shortcut_path(game, &applications_dir()), "menu")
}

pub fn desktop_shortcut_exists(game: &Game) -> bool {
    desktop_dir().is_some_and(|desktop| shortcut_path(game, &desktop).exists())
}

pub fn menu_shortcut_exists(game: &Game) -> bool {
    shortcut_path(game, &applications_dir()).exists()
}

pub fn duplicate_game(game: &Game) -> Result<Game> {
    let new_id = crate::library::generate_id();

    let mut new_game = game.clone();
    new_game.metadata.id = new_id;
    new_game.metadata.name = format!("{} (Copy)", game.metadata.name);
    new_game.metadata.playtime = 0.0;
    new_game.metadata.last_played = String::new();
    new_game.metadata.added = crate::library::rfc3339_now();

    Library::save_game_static(&new_game)?;

    Ok(new_game)
}

pub fn disk_free_space(path: &str) -> u64 {
    let mut p = std::path::Path::new(path).to_path_buf();
    while !p.exists() {
        if !p.pop() {
            return 0;
        }
    }
    match nix::sys::statvfs::statvfs(&p) {
        Ok(stat) => stat.fragment_size() * stat.blocks_available(),
        Err(e) => {
            tracing::error!("statvfs failed for {}: {}", p.display(), e);
            0
        }
    }
}

pub fn show_file_dialog(select_folder: bool, title: &str, default_path: &str) -> String {
    if which::which("zenity").is_ok() {
        let mut cmd = Command::new("zenity");
        if select_folder {
            cmd.arg("--file-selection").arg("--directory");
        } else {
            cmd.arg("--file-selection");
        }
        cmd.arg("--title").arg(title);
        if !default_path.is_empty() {
            cmd.arg("--filename").arg(default_path);
        }

        match cmd.output() {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
            _ => {}
        }
    }

    if which::which("kdialog").is_ok() {
        let mut cmd = Command::new("kdialog");
        if select_folder {
            cmd.arg("--getexistingdirectory");
            if !default_path.is_empty() {
                cmd.arg(default_path);
            }
        } else {
            cmd.arg("--getopenfilename");
            if !default_path.is_empty() {
                cmd.arg(default_path);
            }
        }
        cmd.arg("--title").arg(title);

        match cmd.output() {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
            _ => {}
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_sanitize_slug() {
        assert_eq!(sanitize_slug("Elden Ring"), "elden-ring");
        assert_eq!(sanitize_slug("Game!!!"), "game");
        assert_eq!(sanitize_slug("Test  Game"), "test-game");
        assert_eq!(sanitize_slug("Super Mario 64"), "super-mario-64");
    }

    #[test]
    fn test_desktop_filename() {
        assert_eq!(desktop_filename("elden-ring", "abc123"), "omikuji.elden-ring-abc123.desktop");
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
