use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QModelIndex, QString};

use omikuji_core::library::{Game, Library};
use omikuji_core::media;

impl super::qobject::GameModel {
    pub fn epic_check_existing_install(
        &self,
        app_name: &QString,
        install_path: &QString,
    ) -> QString {
        let app_s = app_name.to_string();
        let install_s = install_path.to_string();
        if app_s.is_empty() || install_s.trim().is_empty() {
            return QString::from(r#"{"bytes":0,"hasResume":false}"#);
        }
        let install = std::path::PathBuf::from(install_s.trim());
        let (bytes, has_resume) = omikuji_core::epic::inspect_existing_install(&app_s, &install);
        QString::from(&format!(
            r#"{{"bytes":{},"hasResume":{}}}"#,
            bytes, has_resume
        ))
    }

    pub fn fetch_epic_install_size(
        self: Pin<&mut Self>,
        request_id: &QString,
        app_name: &QString,
    ) {
        let rid = request_id.to_string();
        let app_name_str = app_name.to_string();

        // os thread + fresh runtime: cant call block_on inside the existing tokio context
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[epic_size] failed to create tokio runtime: {}", e);
                    omikuji_core::install_sizes::push(omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("tokio runtime: {}", e),
                    });
                    return;
                }
            };
            let result = rt.block_on(async {
                omikuji_core::epic::fetch_install_size(&app_name_str).await
            });

            let pushed = match result {
                Ok(size) => omikuji_core::install_sizes::InstallSizeResult {
                    request_id: rid,
                    download_bytes: size.download_bytes,
                    install_bytes: size.install_bytes,
                    error: String::new(),
                },
                Err(e) => {
                    eprintln!("[epic_size] error: {}", e);
                    omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("{}", e),
                    }
                }
            };
            omikuji_core::install_sizes::push(pushed);
        });
    }

    pub fn epic_import_after_install(
        mut self: Pin<&mut Self>,
        app_name: &QString,
        display_name: &QString,
        prefix_path: &QString,
        runner_version: &QString,
    ) -> QString {
        use omikuji_core::library::{
            default_color, GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, SourceConfig, SystemConfig, WineConfig,
        };

        let app_name_s = app_name.to_string();

        if self.library.game.iter().any(|g| g.metadata.id == app_name_s) {
            eprintln!("[epic_import] already in library: {}", app_name_s);
            return QString::from(&app_name_s);
        }

        let Some(info) = omikuji_core::epic::find_installed_info(&app_name_s) else {
            eprintln!("[epic_import] no install info for {} — leaving library alone", app_name_s);
            return QString::default();
        };

        let display_str = display_name.to_string();
        let title = info
            .title
            .clone()
            .unwrap_or_else(|| if display_str.is_empty() { app_name_s.clone() } else { display_str });
        let prefix_str = prefix_path.to_string();
        let runner_str = runner_version.to_string();

        let mut game = Game {
            metadata: Metadata {
                id: app_name_s.clone(),
                name: title.clone(),
                sort_name: String::new(),
                slug: String::new(),
                exe: info.executable.clone(),
                color: default_color(),
                playtime: 0.0,
                last_played: String::new(),
                banner: String::new(),
                coverart: String::new(),
                icon: String::new(),
                favourite: false,
                categories: vec!["Epic Games".to_string()],
            },
            source: SourceConfig {
                kind: "epic".to_string(),
                app_id: app_name_s.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: "wine".to_string(),
            },
            wine: WineConfig {
                version: runner_str,
                prefix: prefix_str,
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("[epic_import] failed to save: {}", e);
            return QString::default();
        }

        let id_for_media = game.metadata.id.clone();
        let name_for_media = game.metadata.name.clone();
        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let id_for_refresh = id_for_media.clone();
            media::fetch_media_blocking_with(&id_for_media, &name_for_media, |_| {
                let id_inner = id_for_refresh.clone();
                let _ = qt_thread.queue(move |mut obj: Pin<&mut super::qobject::GameModel>| {
                    let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                        return;
                    };
                    let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
                    let roles = cxx_qt_lib::QList::<i32>::default();
                    obj.as_mut().data_changed(&idx, &idx, &roles);
                });
            });
        });

        self.as_mut().begin_insert_rows(&QModelIndex::default(), row, row);
        self.as_mut().rust_mut().get_mut().library.game.push(game);
        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        eprintln!("[epic_import] imported '{}' as id '{}'", title, app_name_s);
        QString::from(&app_name_s)
    }

    pub fn epic_uninstall(self: Pin<&mut Self>, game_id: &QString) -> bool {
        let id = game_id.to_string();
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[epic_uninstall] game '{}' not found", id);
            return false;
        };
        if game.source.kind != "epic" || game.source.app_id.is_empty() {
            eprintln!("[epic_uninstall] game '{}' is not an epic entry", id);
            return false;
        }

        let app_id = game.source.app_id.clone();
        let name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let install_path = omikuji_core::epic::find_installed_info(&app_id)
            .map(|i| i.install_path.clone());

        std::thread::spawn(move || {
            let Some(legendary_bin) = omikuji_core::downloads::legendary::find_legendary() else {
                omikuji_core::process::notify_error(
                    omikuji_core::process::ErrorNotification {
                        game_id: game_id_owned.clone(),
                        title: "Uninstall failed".to_string(),
                        message: "`Legendary` not found".to_string(),
                        action: omikuji_core::process::ErrorAction::OpenGlobalSettings,
                    },
                );
                return;
            };

            let entries_to_cancel: Vec<String> = omikuji_core::downloads::manager()
                .list()
                .iter()
                .filter(|e| e.app_id == app_id)
                .map(|e| e.id.clone())
                .collect();
            for entry_id in entries_to_cancel {
                omikuji_core::downloads::manager().cancel(&entry_id);
            }

            omikuji_core::notifications::info(&name, "Uninstalling via Legendary...");

            let result = std::process::Command::new(&legendary_bin)
                .arg("-y")
                .arg("uninstall")
                .arg(&app_id)
                .output();

            match result {
                Ok(out) if out.status.success() => {
                    if let Some(path) = &install_path
                        && path.exists() {
                            eprintln!("[epic_uninstall] legendary exited 0 but {} still exists, forcing cleanup", path.display());
                            omikuji_core::downloads::cleanup_install_dir_blocking(path);
                        }
                    if let Ok(mut lib) = omikuji_core::library::Library::load() {
                        let _ = lib.remove_game(&game_id_owned);
                    }
                    omikuji_core::notifications::success(&name, "Uninstalled");
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    omikuji_core::process::notify_error(
                        omikuji_core::process::ErrorNotification {
                            game_id: game_id_owned.clone(),
                            title: "Uninstall failed".to_string(),
                            message: format!("`legendary` returned an error: {}", err.trim()),
                            action: omikuji_core::process::ErrorAction::None,
                        },
                    );
                }
                Err(e) => {
                    omikuji_core::process::notify_error(
                        omikuji_core::process::ErrorNotification {
                            game_id: game_id_owned.clone(),
                            title: "Uninstall failed".to_string(),
                            message: format!("Couldn't run `legendary`: {}", e),
                            action: omikuji_core::process::ErrorAction::None,
                        },
                    );
                }
            }
        });

        true
    }

    pub fn epic_toggle_overlay(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        enable: bool,
    ) -> bool {
        let id = game_id.to_string();
        let Some(idx) = self.library.game.iter().position(|g| g.metadata.id == id) else {
            eprintln!("[epic_overlay] game '{}' not found", id);
            return false;
        };
        if !self.library.game[idx].is_epic() {
            eprintln!("[epic_overlay] game '{}' is not epic", id);
            return false;
        }

        let (game_name, prefix) = {
            let game = &mut self.as_mut().rust_mut().get_mut().library.game[idx];
            game.source.eos_overlay = enable;
            let _ = Library::save_game_static(game);
            (
                game.metadata.name.clone(),
                omikuji_core::launch::resolve_prefix(game),
            )
        };

        let id_for_thread = id;
        std::thread::spawn(move || {
            use omikuji_core::epic::eos_overlay;
            use omikuji_core::notifications as notif;

            let verb = if enable { "Enabling" } else { "Disabling" };
            notif::info("EOS Overlay", format!("{} for {}…", verb, game_name));

            let result = if enable {
                eos_overlay::enable(&prefix)
            } else {
                eos_overlay::disable(&prefix)
            };

            match result {
                Ok(_) => {
                    let verb = if enable { "Enabled" } else { "Disabled" };
                    notif::success("EOS Overlay", format!("{} for {}", verb, game_name));
                }
                Err(e) => {
                    notif::error("EOS Overlay", format!("{} failed: {}", verb, e));
                    // roll back the persisted flag so the ui toggle re-syncs to the real state
                    if let Ok(Some(mut game)) = omikuji_core::library::Library::load_game_by_id(&id_for_thread) {
                        game.source.eos_overlay = !enable;
                        let _ = omikuji_core::library::Library::save_game_static(&game);
                    }
                }
            }
        });

        true
    }

    pub fn epic_overlay_is_installed(&self) -> bool {
        omikuji_core::epic::eos_overlay::is_installed()
    }

    pub fn epic_set_cloud_saves(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        enable: bool,
    ) -> bool {
        let id = game_id.to_string();
        let Some(idx) = self.library.game.iter().position(|g| g.metadata.id == id) else {
            eprintln!("[epic_cloud] game '{}' not found", id);
            return false;
        };
        if !self.library.game[idx].is_epic() {
            eprintln!("[epic_cloud] game '{}' is not epic", id);
            return false;
        }

        // persist the flag first; only probe legendary if save_path is still empty
        let (game_name, should_probe, game_clone) = {
            let game = &mut self.as_mut().rust_mut().get_mut().library.game[idx];
            let needs_probe = enable && game.source.save_path.is_empty();
            game.source.cloud_saves = enable;
            let _ = Library::save_game_static(game);
            (game.metadata.name.clone(), needs_probe, game.clone())
        };

        if !should_probe {
            return true;
        }

        let id_for_thread = id;
        std::thread::spawn(move || {
            use omikuji_core::notifications as notif;

            notif::info(
                "Cloud Saves",
                format!("Discovering save path for {}…", game_name),
            );

            match omikuji_core::epic::discover_save_path(&game_clone) {
                Ok(path) if !path.is_empty() => {
                    if let Ok(Some(mut game)) = omikuji_core::library::Library::load_game_by_id(&id_for_thread) {
                        game.source.save_path = path.clone();
                        let _ = omikuji_core::library::Library::save_game_static(&game);
                    }
                    notif::success("Cloud Saves", format!("Save path resolved: {}", path));
                }
                Ok(_) => {
                    notif::warning(
                        "Cloud Saves",
                        "No cloud save path found — this game may not support Epic cloud saves. You can enter one manually below.",
                    );
                }
                Err(e) => {
                    notif::error("Cloud Saves", format!("Discovery failed: {}", e));
                }
            }
        });

        true
    }
}
