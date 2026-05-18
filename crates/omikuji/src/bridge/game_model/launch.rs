use std::pin::Pin;

use cxx_qt_lib::QString;

use omikuji_core::library::Game;

impl super::qobject::GameModel {
    pub fn launch_game(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("launch_game: invalid index {}", index);
            return false;
        };

        // pre-launch update check. network errors are intentionally swallowed so a hiccup doesnt block the user from playing.
        if game.source.kind == "gacha"
            && let Some(info) = blocking_check_gacha_update(&game.source.app_id) {
                omikuji_core::process::notify_update_required(
                    omikuji_core::process::UpdateNotification {
                        game_id: game.metadata.id.clone(),
                        app_id: game.source.app_id.clone(),
                        from_version: info.from_version,
                        to_version: info.to_version,
                        download_size: info.download_size,
                        can_diff: info.can_diff,
                        delta_supported: info.delta_supported,
                    },
                );
                return false;
            }

        if game.source.kind == "epic"
            && omikuji_core::ui_settings::UiSettings::load().behavior.auto_check_epic_updates_on_launch
            && let Some(info) = omikuji_core::epic::updates::blocking_check_epic_update(&game.source.app_id) {
                omikuji_core::process::notify_update_required(
                    omikuji_core::process::UpdateNotification {
                        game_id: game.metadata.id.clone(),
                        app_id: game.source.app_id.clone(),
                        from_version: info.from_version,
                        to_version: info.to_version,
                        download_size: info.download_size,
                        can_diff: true,
                        delta_supported: true,
                    },
                );
                return false;
            }

        if game.source.kind == "gog"
            && omikuji_core::ui_settings::UiSettings::load().behavior.auto_check_gog_updates_on_launch
            && let Some(info) = omikuji_core::gog::updates::blocking_check_gog_update(&game.source.app_id) {
                omikuji_core::process::notify_update_required(
                    omikuji_core::process::UpdateNotification {
                        game_id: game.metadata.id.clone(),
                        app_id: game.source.app_id.clone(),
                        from_version: info.from_version,
                        to_version: info.to_version,
                        download_size: info.download_size,
                        can_diff: true,
                        delta_supported: true,
                    },
                );
                return false;
            }

        self.try_spawn_launch(game)
    }

    pub fn launch_game_force(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("launch_game_force: invalid index {}", index);
            return false;
        };
        self.try_spawn_launch(game)
    }

    fn try_spawn_launch(&self, game: &Game) -> bool {
        if omikuji_core::process::is_game_running(&game.metadata.id) {
            eprintln!("game '{}' is already running", game.metadata.name);
            return false;
        }

        if game.runner.runner_type == "steam" {
            omikuji_core::notifications::info(
                &game.metadata.name,
                "Launching through Steam... any errors will show in Steam itself",
            );
        }

        match omikuji_core::launch::build_launch(game) {
            Ok(config) => {
                eprintln!("launching '{}': {:?}", game.metadata.name, config.command);

                // spawn os thread + build fresh runtime: we're already inside the #[tokio::main] runtime, cant block_on from here directly
                let game_name = game.metadata.name.clone();
                let game_id = game.metadata.id.clone();
                let logs_dir = omikuji_core::logs_dir();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match omikuji_core::process::launch_game(&config).await {
                            Ok(proc_id) => {
                                eprintln!("game '{}' launched, process id: {:?}", game_name, proc_id);
                                eprintln!("logs: {}", logs_dir.display());
                            }
                            Err(e) => {
                                eprintln!("failed to launch '{}': {}", game_name, e);
                                omikuji_core::process::notify_error(
                                    omikuji_core::process::ErrorNotification {
                                        game_id,
                                        title: "Couldn't launch".to_string(),
                                        message: e.to_string(),
                                        action: omikuji_core::process::ErrorAction::OpenGameSettings,
                                    },
                                );
                            }
                        }
                    });
                });

                true
            }
            Err(e) => {
                eprintln!("failed to build launch config: {}", e);
                let action = if e.downcast_ref::<omikuji_core::launch::ComponentMissing>().is_some() {
                    omikuji_core::process::ErrorAction::OpenGlobalSettings
                } else {
                    omikuji_core::process::ErrorAction::OpenGameSettings
                };
                omikuji_core::process::notify_error(
                    omikuji_core::process::ErrorNotification {
                        game_id: game.metadata.id.clone(),
                        title: "Couldn't launch".to_string(),
                        message: e.to_string(),
                        action,
                    },
                );
                false
            }
        }
    }

    pub fn stop_game(&self, game_id: &QString) {
        let id = game_id.to_string();
        eprintln!("[stop_game] requesting stop for game '{}'", id);
        omikuji_core::process::stop_game(&id);
    }

    // tool name must match one of the WineTool enum arms below; unknown names are dropped
    pub fn run_wine_tool(&self, game_id: &QString, tool: &QString) {
        let id = game_id.to_string();
        let tool_name = tool.to_string();
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[run_wine_tool] game '{}' not found", id);
            return;
        };
        let t = match tool_name.as_str() {
            "winecfg" => omikuji_core::wine_tools::WineTool::Winecfg,
            "winetricks" => omikuji_core::wine_tools::WineTool::Winetricks,
            "regedit" => omikuji_core::wine_tools::WineTool::Regedit,
            "cmd" => omikuji_core::wine_tools::WineTool::Cmd,
            "winefile" => omikuji_core::wine_tools::WineTool::Winefile,
            "killwineserver" => omikuji_core::wine_tools::WineTool::KillWineserver,
            other => {
                eprintln!("[run_wine_tool] unknown tool '{}'", other);
                return;
            }
        };
        let display_name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let tool_label = tool_name.clone();
        // prefix-init and umu-run startup can be slow, detach so the ui doesnt block
        std::thread::spawn(move || match omikuji_core::wine_tools::run(&game, t) {
            Ok(_child) => {
                omikuji_core::notifications::info(&display_name, format!("Opened {}", tool_label));
            }
            Err(e) => {
                omikuji_core::process::notify_error(
                    omikuji_core::process::ErrorNotification {
                        game_id: game_id_owned,
                        title: format!("{} failed", tool_label),
                        message: format!("{}", e),
                        action: omikuji_core::process::ErrorAction::OpenGameSettings,
                    },
                );
            }
        });
    }

    pub fn run_wine_exe(&self, game_id: &QString, exe_path: &QString) {
        let id = game_id.to_string();
        let exe = exe_path.to_string();
        if exe.is_empty() {
            return;
        }
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[run_wine_exe] game '{}' not found", id);
            return;
        };
        let display_name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let path = std::path::PathBuf::from(&exe);
        let file_label = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| exe.clone());
        std::thread::spawn(move || {
            match omikuji_core::wine_tools::run(&game, omikuji_core::wine_tools::WineTool::RunExe(path)) {
                Ok(_child) => {
                    omikuji_core::notifications::info(
                        &display_name,
                        format!("Running {}", file_label),
                    );
                }
                Err(e) => {
                    omikuji_core::process::notify_error(
                        omikuji_core::process::ErrorNotification {
                            game_id: game_id_owned,
                            title: "Couldn't run executable".to_string(),
                            message: format!("`{}` failed: {}", file_label, e),
                            action: omikuji_core::process::ErrorAction::OpenGameSettings,
                        },
                    );
                }
            }
        });
    }

    pub fn check_exited_games(mut self: Pin<&mut Self>) {
        for game_id in omikuji_core::process::take_exited_games() {
            self.as_mut().game_stopped(&QString::from(&game_id));
        }
    }

    pub fn drain_game_log_events(mut self: Pin<&mut Self>) {
        for id in omikuji_core::game_logs::drain_dirty() {
            self.as_mut().game_log_appended(&QString::from(&id));
        }
    }

    pub fn game_log(&self, game_id: &QString) -> QString {
        QString::from(&omikuji_core::game_logs::get_log(&game_id.to_string()))
    }

    pub fn clear_game_log(&self, game_id: &QString) {
        omikuji_core::game_logs::clear_log(&game_id.to_string());
    }

    pub fn save_game_log(&self, game_id: &QString) -> QString {
        let id = game_id.to_string();
        let body = omikuji_core::game_logs::get_log(&id);
        if body.is_empty() {
            return QString::from("");
        }
        let dir = omikuji_core::logs_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[game_log] couldn't create {}: {}", dir.display(), e);
            return QString::from("");
        }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let file = dir.join(format!("{}_{}.log", id, ts));
        match std::fs::write(&file, body) {
            Ok(_) => QString::from(file.to_string_lossy().as_ref()),
            Err(e) => {
                eprintln!("[game_log] write {} failed: {}", file.display(), e);
                QString::from("")
            }
        }
    }

    pub fn launch_console_mode(&self) {
        omikuji_core::ui_settings::UiSettings::set_console_mode_active(true);
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe)
                .arg("console")
                .env("OMIKUJI_BYPASS_SINGLE_INSTANCE", "1")
                .spawn();
        }
        std::process::exit(0);
    }

    pub fn launch_desktop_mode(&self) {
        omikuji_core::ui_settings::UiSettings::set_console_mode_active(false);
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe)
                .env("OMIKUJI_BYPASS_SINGLE_INSTANCE", "1")
                .spawn();
        }
        std::process::exit(0);
    }
}

// flattened update info passed from gacha backends to the launch hook
struct GachaUpdateInfo {
    from_version: String,
    to_version: String,
    download_size: u64,
    can_diff: bool,
    delta_supported: bool,
}

// launch_game is called from the Qt event loop, which already runs inside the
// #[tokio::main] runtime. building a second runtime on that thread panics
// ("cannot start a runtime from within a runtime"). a plain os thread gives us a clean context to block_on from
fn blocking_check_gacha_update(app_id: &str) -> Option<GachaUpdateInfo> {
    let aid = app_id.to_string();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[launch] update check: runtime build failed: {}", e);
                return None;
            }
        };
        rt.block_on(async {
            let (manifest, edition_id, _voices) =
                omikuji_core::gachas::strategies::find_for_app_id(&aid)?;
            let info =
                omikuji_core::gachas::strategies::check_for_update(&manifest, &edition_id).await?;
            Some(GachaUpdateInfo {
                from_version: info.from_version,
                to_version: info.to_version,
                download_size: info.download_size,
                can_diff: info.can_diff,
                delta_supported: info.delta_supported,
            })
        })
    })
    .join()
    .ok()
    .flatten()
}
