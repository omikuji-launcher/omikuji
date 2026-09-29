use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use omikuji_core::defaults::Defaults;
use omikuji_core::library::{Game, Library, SourceKind};
use omikuji_core::store::nile;
use omikuji_core::{install_sizes, media};

use crate::bridge::expand_path;

impl super::qobject::GameModel {
    pub fn nile_check_existing_install(
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
        let (bytes, has_resume) = nile::inspect_existing_install(&app_s, &install);
        QString::from(&format!(
            r#"{{"bytes":{},"hasResume":{}}}"#,
            bytes, has_resume
        ))
    }

    pub fn nile_dir_has_game(&self, _app_id: &QString, install_path: &QString) -> bool {
        let path = expand_path(install_path);
        if path.trim().is_empty() {
            return false;
        }
        nile::fuel::path_in(std::path::Path::new(path.trim())).exists()
    }

    pub fn fetch_nile_install_size(self: Pin<&mut Self>, request_id: &QString, app_name: &QString) {
        let rid = request_id.to_string();
        let app_name_str = app_name.to_string();

        install_sizes::spawn_fetch_ex(rid, move || async move {
            nile::fetch_install_size(&app_name_str)
                .await
                .map(|s| {
                    (
                        s.download_bytes,
                        s.download_bytes,
                        String::new(),
                        "[]".to_string(),
                    )
                })
                .map_err(|e| e.to_string())
        });
    }

    pub fn nile_import_after_install(
        mut self: Pin<&mut Self>,
        app_name: &QString,
        display_name: &QString,
        prefix_path: &QString,
        runner_version: &QString,
    ) -> QString {
        use omikuji_core::library::{
            GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, RunnerType, SourceConfig,
            SystemConfig, WineConfig,
        };

        let app_name_s = app_name.to_string();

        if self
            .library
            .game
            .iter()
            .any(|g| g.metadata.id == app_name_s)
        {
            tracing::info!("already in library: {}", app_name_s);
            return QString::from(&app_name_s);
        }

        let Some(info) = nile::find_installed_info(&app_name_s) else {
            tracing::warn!("no install info for {} - leaving library alone", app_name_s);
            return QString::default();
        };

        if info.executable.as_os_str().is_empty() {
            tracing::warn!(
                "no fuel.json exe for {} - leaving library alone",
                app_name_s
            );
            return QString::default();
        }

        let display_str = display_name.to_string();
        let title = if display_str.is_empty() {
            app_name_s.clone()
        } else {
            display_str
        };

        let fuel = nile::fuel::load(&info.install_path).ok();

        let mut game = Game {
            metadata: Metadata {
                categories: vec!["Amazon".to_string()],
                ..Metadata::new(app_name_s.clone(), title.clone(), info.executable.clone())
            },
            source: SourceConfig {
                kind: SourceKind::Nile,
                app_id: app_name_s.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: RunnerType::Wine,
            },
            wine: WineConfig {
                version: runner_version.to_string(),
                prefix: prefix_path.to_string(),
                ..WineConfig::default()
            },
            launch: LaunchConfig {
                args: fuel
                    .as_ref()
                    .map(|f| f.main.args.clone())
                    .unwrap_or_default(),
                working_dir: fuel
                    .as_ref()
                    .and_then(|f| f.working_dir_override(&info.install_path))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                ..LaunchConfig::default()
            },
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
}
