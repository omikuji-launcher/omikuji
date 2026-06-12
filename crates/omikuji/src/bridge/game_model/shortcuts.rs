impl super::qobject::GameModel {
    pub fn browse_files(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            tracing::warn!("browse_files: invalid index {}", index);
            return false;
        };

        let Some(dir) = omikuji_core::desktop::get_game_browse_dir(game) else {
            tracing::warn!("browse_files: no directory for game '{}'", game.metadata.name);
            return false;
        };

        match omikuji_core::desktop::browse_files(&dir) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("browse_files failed: {}", e);
                false
            }
        }
    }

    pub fn create_desktop_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            tracing::warn!("create_desktop_shortcut: invalid index {}", index);
            return false;
        };

        match omikuji_core::desktop::create_desktop_shortcut(game) {
            Ok(path) => {
                tracing::info!("created desktop shortcut: {}", path.display());
                true
            }
            Err(e) => {
                tracing::error!("create_desktop_shortcut failed: {}", e);
                false
            }
        }
    }

    pub fn create_menu_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            tracing::warn!("create_menu_shortcut: invalid index {}", index);
            return false;
        };

        match omikuji_core::desktop::create_menu_shortcut(game) {
            Ok(path) => {
                tracing::info!("created menu shortcut: {}", path.display());
                true
            }
            Err(e) => {
                tracing::error!("create_menu_shortcut failed: {}", e);
                false
            }
        }
    }

    pub fn remove_desktop_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };

        match omikuji_core::desktop::remove_desktop_shortcut(game) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("remove_desktop_shortcut failed: {}", e);
                false
            }
        }
    }

    pub fn remove_menu_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };

        match omikuji_core::desktop::remove_menu_shortcut(game) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("remove_menu_shortcut failed: {}", e);
                false
            }
        }
    }

    pub fn has_desktop_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };
        omikuji_core::desktop::desktop_shortcut_exists(game)
    }

    pub fn has_menu_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };
        omikuji_core::desktop::menu_shortcut_exists(game)
    }

    pub fn create_steam_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            tracing::warn!("create_steam_shortcut: invalid index {}", index);
            return false;
        };

        match omikuji_core::steam::shortcuts::create_shortcut(game) {
            Ok(path) => {
                tracing::info!("created steam shortcut in {}", path.display());
                true
            }
            Err(e) => {
                tracing::error!("create_steam_shortcut failed: {}", e);
                false
            }
        }
    }

    pub fn remove_steam_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };

        match omikuji_core::steam::shortcuts::remove_shortcut(game) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("remove_steam_shortcut failed: {}", e);
                false
            }
        }
    }

    pub fn has_steam_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };
        omikuji_core::steam::shortcuts::shortcut_exists(game)
    }

    pub fn steam_shortcut_available(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };
        game.runner.runner_type != "steam" && omikuji_core::steam::shortcuts::available()
    }
}
