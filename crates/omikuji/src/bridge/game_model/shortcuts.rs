use omikuji_core::desktop;
use omikuji_core::store::steam::shortcuts as steam;

use super::ok_bool;

impl super::qobject::GameModel {
    pub fn browse_files(&self, index: i32) -> bool {
        self.game_at(index)
            .and_then(desktop::get_game_browse_dir)
            .is_some_and(|dir| ok_bool("browse_files", desktop::browse_files(&dir)))
    }

    pub fn create_desktop_shortcut(&self, index: i32) -> bool {
        self.game_at(index).is_some_and(|game| {
            ok_bool(
                "create_desktop_shortcut",
                desktop::create_desktop_shortcut(game)
                    .inspect(|p| tracing::info!("created desktop shortcut: {}", p.display())),
            )
        })
    }

    pub fn create_menu_shortcut(&self, index: i32) -> bool {
        self.game_at(index).is_some_and(|game| {
            ok_bool(
                "create_menu_shortcut",
                desktop::create_menu_shortcut(game)
                    .inspect(|p| tracing::info!("created menu shortcut: {}", p.display())),
            )
        })
    }

    pub fn remove_desktop_shortcut(&self, index: i32) -> bool {
        self.game_at(index).is_some_and(|game| {
            ok_bool(
                "remove_desktop_shortcut",
                desktop::remove_desktop_shortcut(game),
            )
        })
    }

    pub fn remove_menu_shortcut(&self, index: i32) -> bool {
        self.game_at(index).is_some_and(|game| {
            ok_bool("remove_menu_shortcut", desktop::remove_menu_shortcut(game))
        })
    }

    pub fn has_desktop_shortcut(&self, index: i32) -> bool {
        self.game_at(index)
            .is_some_and(desktop::desktop_shortcut_exists)
    }

    pub fn has_menu_shortcut(&self, index: i32) -> bool {
        self.game_at(index)
            .is_some_and(desktop::menu_shortcut_exists)
    }

    pub fn create_steam_shortcut(&self, index: i32) -> bool {
        self.game_at(index).is_some_and(|game| {
            ok_bool(
                "create_steam_shortcut",
                steam::create_shortcut(game)
                    .inspect(|p| tracing::info!("created steam shortcut in {}", p.display())),
            )
        })
    }

    pub fn remove_steam_shortcut(&self, index: i32) -> bool {
        self.game_at(index)
            .is_some_and(|game| ok_bool("remove_steam_shortcut", steam::remove_shortcut(game)))
    }

    pub fn has_steam_shortcut(&self, index: i32) -> bool {
        self.game_at(index).is_some_and(steam::shortcut_exists)
    }

    pub fn steam_shortcut_available(&self, index: i32) -> bool {
        self.game_at(index)
            .is_some_and(|game| game.runner.runner_type != "steam" && steam::available())
    }
}
