use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use omikuji_core::defaults::Defaults;
use omikuji_core::library::{Game, Library, SourceKind};
use omikuji_core::store::epic;
use omikuji_core::{install_sizes, launch, media, notifications};

use crate::bridge::expand_path;

impl super::qobject::GameModel {
    pub fn epic_check_existing_install(
        &self,
        app_name: &QString,
        install_path: &QString,
    ) -> QString {
        let app_s = app_name.to_string();
        let install_s = expand_path(install_path);
        if app_s.is_empty() || install_s.trim().is_empty() {
            return QString::from(r#"{"bytes":0,"hasResume":false}"#);
        }
        let install = std::path::PathBuf::from(install_s.trim());
        let (bytes, has_resume) = epic::inspect_existing_install(&app_s, &install);
        QString::from(&format!(
            r#"{{"bytes":{},"hasResume":{}}}"#,
            bytes, has_resume
        ))
    }

    pub fn fetch_epic_game_details(self: Pin<&mut Self>, request_id: &QString, app_name: &QString) {
        let rid = request_id.to_string();
        let app_name_str = app_name.to_string();

        install_sizes::spawn_fetch_details(rid, move || async move {
            epic::fetch_game_details(&app_name_str)
                .await
                .map_err(|e| e.to_string())
        });
    }

    pub fn fetch_epic_install_size(self: Pin<&mut Self>, request_id: &QString, app_name: &QString) {
        let rid = request_id.to_string();
        let app_name_str = app_name.to_string();

        install_sizes::spawn_fetch_ex(rid, move || async move {
            epic::fetch_install_size(&app_name_str)
                .await
                .map(|s| {
                    let dlcs = serde_json::to_string(&s.dlcs).unwrap_or_else(|_| "[]".to_string());
                    (s.download_bytes, s.install_bytes, s.launch_exe, dlcs)
                })
                .map_err(|e| e.to_string())
        });
    }

    pub fn epic_dir_has_game(&self, launch_exe: &QString, install_path: &QString) -> bool {
        let path = expand_path(install_path);
        if path.trim().is_empty() {
            return false;
        }
        let dir = std::path::Path::new(path.trim());
        let exe = launch_exe.to_string();
        if !exe.is_empty() {
            return dir.join(&exe).exists();
        }
        dir.join(".egstore").exists()
    }

    pub fn epic_import_after_install(
        mut self: Pin<&mut Self>,
        app_name: &QString,
        display_name: &QString,
        prefix_path: &QString,
        runner_version: &QString,
        dlcs: &QString,
    ) -> QString {
        use omikuji_core::library::{
            GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, RunnerType, SourceConfig,
            SystemConfig, WineConfig,
        };

        let app_name_s = app_name.to_string();

        if self.library.game(&app_name_s).is_some() {
            tracing::info!("already in library: {}", app_name_s);
            return QString::from(&app_name_s);
        }

        let Some(info) = epic::find_installed_info(&app_name_s) else {
            tracing::warn!("no install info for {} - leaving library alone", app_name_s);
            return QString::default();
        };

        let display_str = display_name.to_string();
        let title = info.title.clone().unwrap_or_else(|| {
            if display_str.is_empty() {
                app_name_s.clone()
            } else {
                display_str
            }
        });
        let prefix_str = prefix_path.to_string();
        let runner_str = runner_version.to_string();

        let mut game = Game {
            metadata: Metadata {
                categories: vec!["Epic Games".to_string()],
                ..Metadata::new(app_name_s.clone(), title.clone(), info.executable.clone())
            },
            source: SourceConfig {
                kind: SourceKind::Epic,
                app_id: app_name_s.clone(),
                dlcs: serde_json::from_str(&dlcs.to_string()).unwrap_or_default(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: RunnerType::Wine,
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
        game.seed_from_defaults(&Defaults::load());

        if let Err(e) = Library::save_game_static(&game) {
            tracing::error!("failed to save: {}", e);
            return QString::default();
        }

        let id_for_media = game.metadata.id.clone();
        let name_for_media = game.metadata.name.clone();
        let qt_thread = self.as_mut().qt_thread();
        let on_asset = super::media_changed_notifier(qt_thread, id_for_media.clone());
        std::thread::spawn(move || {
            media::fetch_media_blocking_with(
                media::MediaSlot::Live,
                &id_for_media,
                &name_for_media,
                on_asset,
            );
        });

        self.as_mut().insert_game_sorted(game);

        tracing::info!("imported '{}' as id '{}'", title, app_name_s);
        QString::from(&app_name_s)
    }

    fn is_epic_game(&self, id: &str) -> bool {
        match self.library.game(id) {
            Some(game) if game.is_epic() => true,
            Some(_) => {
                tracing::warn!("game '{}' is not epic", id);
                false
            }
            None => {
                tracing::warn!("game '{}' not found", id);
                false
            }
        }
    }

    pub fn epic_toggle_overlay(mut self: Pin<&mut Self>, game_id: &QString, enable: bool) -> bool {
        let id = game_id.to_string();
        if !self.is_epic_game(&id) {
            return false;
        }
        let Some(game) = self
            .as_mut()
            .update_game(&id, |g| g.source.eos_overlay = enable)
        else {
            return false;
        };
        let game_name = game.metadata.name.clone();
        let prefix = launch::resolve_prefix(&game);

        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            use omikuji_core::store::epic::eos_overlay;

            let verb = if enable { "Enabling" } else { "Disabling" };
            notifications::info("EOS Overlay", format!("{} for {}…", verb, game_name));

            let result = if enable {
                eos_overlay::enable(&prefix)
            } else {
                eos_overlay::disable(&prefix)
            };

            match result {
                Ok(_) => {
                    let verb = if enable { "Enabled" } else { "Disabled" };
                    notifications::success("EOS Overlay", format!("{} for {}", verb, game_name));
                }
                Err(e) => {
                    notifications::error("EOS Overlay", format!("{} failed: {}", verb, e));
                    // roll back the persisted flag so the ui toggle re-syncs to the real state
                    let _ = qt_thread.queue(move |obj: Pin<&mut super::qobject::GameModel>| {
                        obj.update_game(&id, |g| g.source.eos_overlay = !enable);
                    });
                }
            }
        });

        true
    }

    pub fn epic_overlay_is_installed(&self) -> bool {
        epic::eos_overlay::is_installed()
    }

    pub fn epic_set_cloud_saves(mut self: Pin<&mut Self>, game_id: &QString, enable: bool) -> bool {
        let id = game_id.to_string();
        if !self.is_epic_game(&id) {
            return false;
        }

        // persist the flag first; only probe legendary if save_path is still empty
        let Some(game) = self
            .as_mut()
            .update_game(&id, |g| g.source.cloud_saves = enable)
        else {
            return false;
        };
        if !enable || !game.source.save_path.is_empty() {
            return true;
        }

        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            notifications::info(
                "Cloud Saves",
                format!("Discovering save path for {}…", game.metadata.name),
            );

            match epic::discover_save_path(&game) {
                Ok(path) if !path.is_empty() => {
                    notifications::success("Cloud Saves", format!("Save path resolved: {}", path));
                    let _ = qt_thread.queue(move |obj: Pin<&mut super::qobject::GameModel>| {
                        obj.update_game(&id, |g| g.source.save_path = path);
                    });
                }
                Ok(_) => {
                    notifications::warning(
                        "Cloud Saves",
                        "No cloud save path found. Enter one manually below if the game supports Epic cloud saves.",
                    );
                }
                Err(e) => {
                    notifications::error("Cloud Saves", format!("Discovery failed: {}", e));
                }
            }
        });

        true
    }
}
