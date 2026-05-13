use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QModelIndex, QString};

use omikuji_core::library::{Game, Library};
use omikuji_core::media;

impl super::qobject::GameModel {
    pub fn gog_check_existing_install(
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
        let (bytes, has_resume) = omikuji_core::gog::inspect_existing_install(&app_s, &install);
        QString::from(&format!(
            r#"{{"bytes":{},"hasResume":{}}}"#,
            bytes, has_resume
        ))
    }

    pub fn fetch_gog_install_size(
        self: Pin<&mut Self>,
        request_id: &QString,
        app_name: &QString,
    ) {
        let rid = request_id.to_string();
        let app_name_str = app_name.to_string();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[gog_size] failed to create tokio runtime: {}", e);
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
                omikuji_core::gog::fetch_install_size(&app_name_str).await
            });

            let pushed = match result {
                Ok(size) => omikuji_core::install_sizes::InstallSizeResult {
                    request_id: rid,
                    download_bytes: size.download_bytes,
                    install_bytes: size.install_bytes,
                    error: String::new(),
                },
                Err(e) => {
                    eprintln!("[gog_size] error: {}", e);
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

    pub fn gog_import_after_install(
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
            eprintln!("[gog_import] already in library: {}", app_name_s);
            return QString::from(&app_name_s);
        }

        let Some(info) = omikuji_core::gog::find_installed_info(&app_name_s) else {
            eprintln!("[gog_import] no install info for {} — leaving library alone", app_name_s);
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
                categories: vec!["GOG".to_string()],
            },
            source: SourceConfig {
                kind: "gog".to_string(),
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
            eprintln!("[gog_import] failed to save: {}", e);
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

        eprintln!("[gog_import] imported '{}' as id '{}'", title, app_name_s);
        QString::from(&app_name_s)
    }

    pub fn gog_uninstall(self: Pin<&mut Self>, game_id: &QString) -> bool {
        let id = game_id.to_string();
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[gog_uninstall] game '{}' not found", id);
            return false;
        };
        if game.source.kind != "gog" || game.source.app_id.is_empty() {
            eprintln!("[gog_uninstall] game '{}' is not a gog entry", id);
            return false;
        }

        let app_id = game.source.app_id.clone();
        let name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let install_path = omikuji_core::gog::find_installed_info(&app_id)
            .map(|i| i.install_path.clone());

        std::thread::spawn(move || {
            let entries_to_cancel: Vec<String> = omikuji_core::downloads::manager()
                .list()
                .iter()
                .filter(|e| e.app_id == app_id)
                .map(|e| e.id.clone())
                .collect();
            for entry_id in entries_to_cancel {
                omikuji_core::downloads::manager().cancel(&entry_id);
            }

            omikuji_core::notifications::info(&name, "Removing GOG game...");
            if let Some(path) = install_path
                && path.exists()
                    && let Err(e) = std::fs::remove_dir_all(&path) {
                        omikuji_core::process::notify_error(
                            omikuji_core::process::ErrorNotification {
                                game_id: game_id_owned.clone(),
                                title: "Uninstall failed".to_string(),
                                message: format!("Failed to remove install dir: {}", e),
                                action: omikuji_core::process::ErrorAction::None,
                            },
                        );
                        return;
                    }
            if let Err(e) = omikuji_core::gog::remove_install(&app_id) {
                eprintln!("[gog_uninstall] registry remove failed: {}", e);
            }
            if let Ok(mut lib) = omikuji_core::library::Library::load() {
                let _ = lib.remove_game(&game_id_owned);
            }
            omikuji_core::notifications::success(&name, "Uninstalled");
        });

        true
    }
}
