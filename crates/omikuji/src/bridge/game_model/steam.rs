use std::path::PathBuf;
use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use omikuji_core::library::{Game, Library};
use omikuji_core::media;

impl super::qobject::GameModel {
    pub fn steam_get_installed_games(&self) -> QString {
        let games = omikuji_core::store::steam::get_installed_games();
        let json_games: Vec<serde_json::Value> = games
            .iter()
            .map(|g| {
                serde_json::json!({
                    "appid": g.appid,
                    "name": g.name,
                    "is_installed": g.is_installed()
                })
            })
            .collect();

        match serde_json::to_string(&json_games) {
            Ok(json) => QString::from(&json),
            Err(_) => QString::from("[]"),
        }
    }

    pub fn steam_local_library_image(&self, appid: &QString) -> QString {
        let appid_str = appid.to_string();
        match omikuji_core::store::steam::local::find_local_library_image(&appid_str) {
            Some(path) => QString::from(&*path.to_string_lossy()),
            None => QString::default(),
        }
    }

    pub fn steam_import_game(mut self: Pin<&mut Self>, appid: &QString, name: &QString) -> bool {
        let appid_str = appid.to_string();
        let name_str = name.to_string();

        tracing::info!("importing {} - {}", appid_str, name_str);

        let already_imported = self.library.game.iter().any(|g| g.metadata.id == appid_str);

        if already_imported {
            tracing::info!("already imported: {}", appid_str);
            return true;
        }

        use omikuji_core::library::{
            GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, SourceConfig, SystemConfig,
            WineConfig,
        };

        let mut game = Game {
            metadata: Metadata::new(appid_str.clone(), name_str.clone(), PathBuf::new()),
            source: SourceConfig {
                kind: "steam".to_string(),
                app_id: appid_str.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: "steam".to_string(),
            },
            wine: WineConfig {
                version: format!("steam:{}", appid_str),
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        if let Err(e) = Library::save_game_static(&game) {
            tracing::error!("failed to save game: {}", e);
            return false;
        }

        let appid_for_media = appid_str.clone();
        let qt_thread = self.as_mut().qt_thread();
        let on_asset = super::media_changed_notifier(qt_thread, appid_str.clone());
        std::thread::spawn(move || {
            let result = media::fetch_steam_media_blocking_with(
                media::MediaSlot::Live,
                &appid_for_media,
                on_asset,
            );
            if result.banner.is_none() && result.coverart.is_none() {
                tracing::warn!("no steam media found for appid {}", appid_for_media);
            }
        });

        self.as_mut().insert_game_sorted(game);

        tracing::info!("imported '{}' (steam appid: {})", name_str, appid_str);
        true
    }

    pub fn steam_sync_playtime(mut self: Pin<&mut Self>) {
        let api_key = omikuji_core::settings::get().steam.api_key.clone();
        if api_key.is_empty() {
            return;
        }

        tracing::info!("syncing playtime from steam api...");
        let qt_thread = self.as_mut().qt_thread();

        // blocking reqwest inside #[tokio::main] panics; escape to an os thread, then marshal the mutation back via qt_thread.queue
        std::thread::spawn(move || {
            let fetch_result = omikuji_core::store::steam::fetch_playtime_data(&api_key);

            let _ = qt_thread.queue(move |mut obj: Pin<&mut super::qobject::GameModel>| {
                let steam_data = match fetch_result {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("steam sync failed: {}", e);
                        return;
                    }
                };

                let library = &mut obj.as_mut().rust_mut().get_mut().library;
                let (updated, total) =
                    omikuji_core::store::steam::apply_playtime_data(library, &steam_data);
                tracing::info!("updated {}/{} steam games", updated, total);

                let mut saved = 0;
                for game in &library.game {
                    if game.runner.runner_type == "steam" {
                        if let Err(e) = Library::save_game_static(game) {
                            tracing::error!("failed to save {}: {}", game.metadata.id, e);
                        } else {
                            saved += 1;
                        }
                    }
                }
                tracing::info!("saved {} games to disk", saved);

                obj.as_mut().begin_reset_model();
                obj.as_mut().end_reset_model();
            });
        });
    }
}
