use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

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

        omikuji_core::install_sizes::spawn_fetch(rid, move || async move {
            omikuji_core::gog::fetch_install_size(&app_name_str)
                .await
                .map(|s| (s.download_bytes, s.install_bytes))
                .map_err(|e| e.to_string())
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
            GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, SourceConfig, SystemConfig, WineConfig,
        };

        let app_name_s = app_name.to_string();

        if self.library.game.iter().any(|g| g.metadata.id == app_name_s) {
            tracing::info!("already in library: {}", app_name_s);
            return QString::from(&app_name_s);
        }

        let Some(info) = omikuji_core::gog::find_installed_info(&app_name_s) else {
            tracing::warn!("no install info for {} - leaving library alone", app_name_s);
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
                categories: vec!["GOG".to_string()],
                ..Metadata::new(app_name_s.clone(), title.clone(), info.executable.clone())
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

        if let Err(e) = Library::save_game_static(&game) {
            tracing::error!("failed to save: {}", e);
            return QString::default();
        }

        let id_for_media = game.metadata.id.clone();
        let name_for_media = game.metadata.name.clone();
        let qt_thread = self.as_mut().qt_thread();
        let on_asset = super::media_changed_notifier(qt_thread, id_for_media.clone());
        std::thread::spawn(move || {
            media::fetch_media_blocking_with(&id_for_media, &name_for_media, on_asset);
        });

        self.as_mut().insert_game_sorted(game);

        tracing::info!("imported '{}' as id '{}'", title, app_name_s);
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
            tracing::error!("game '{}' not found", id);
            return false;
        };
        if game.source.kind != "gog" || game.source.app_id.is_empty() {
            tracing::error!("game '{}' is not a gog entry", id);
            return false;
        }

        let app_id = game.source.app_id.clone();
        let name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let installed = omikuji_core::gog::find_installed_info(&app_id);
        let wrapper_name = omikuji_core::gog::install_wrapper_dir_name(
            installed
                .as_ref()
                .and_then(|i| i.title.as_deref())
                .unwrap_or(&name),
        );
        let install_path = installed.map(|i| i.install_path);

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
                && path.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&path) {
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
                    if !wrapper_name.is_empty()
                        && let Some(parent) = path.parent()
                            && parent.file_name().is_some_and(|n| n.to_string_lossy() == wrapper_name) {
                                let _ = std::fs::remove_dir(parent);
                            }
                }
            if let Err(e) = omikuji_core::gog::remove_install(&app_id) {
                tracing::error!("registry remove failed: {}", e);
            }
            if let Err(e) = omikuji_core::library::Library::remove_game_file(&game_id_owned) {
                tracing::error!("failed to remove game file: {}", e);
            }
            omikuji_core::notifications::success(&name, "Uninstalled");
        });

        true
    }
}
