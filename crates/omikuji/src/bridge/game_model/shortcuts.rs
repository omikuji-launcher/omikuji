impl super::qobject::GameModel {
    pub fn browse_files(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("browse_files: invalid index {}", index);
            return false;
        };

        let Some(dir) = omikuji_core::desktop::get_game_browse_dir(game) else {
            eprintln!("browse_files: no directory for game '{}'", game.metadata.name);
            return false;
        };

        match omikuji_core::desktop::browse_files(&dir) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("browse_files failed: {}", e);
                false
            }
        }
    }

    pub fn create_desktop_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("create_desktop_shortcut: invalid index {}", index);
            return false;
        };

        match omikuji_core::desktop::create_desktop_shortcut(game) {
            Ok(path) => {
                eprintln!("created desktop shortcut: {}", path.display());
                true
            }
            Err(e) => {
                eprintln!("create_desktop_shortcut failed: {}", e);
                false
            }
        }
    }

    pub fn create_menu_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("create_menu_shortcut: invalid index {}", index);
            return false;
        };

        match omikuji_core::desktop::create_menu_shortcut(game) {
            Ok(path) => {
                eprintln!("created menu shortcut: {}", path.display());
                true
            }
            Err(e) => {
                eprintln!("create_menu_shortcut failed: {}", e);
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
                eprintln!("remove_desktop_shortcut failed: {}", e);
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
                eprintln!("remove_menu_shortcut failed: {}", e);
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
}
