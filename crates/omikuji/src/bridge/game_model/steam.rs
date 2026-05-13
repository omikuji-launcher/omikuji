use std::path::PathBuf;
use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QModelIndex, QString};

use omikuji_core::library::{Game, Library};
use omikuji_core::media;

impl super::qobject::GameModel {
    pub fn steam_is_installed(&self) -> bool {
        omikuji_core::steam::is_steam_installed()
    }

    pub fn steam_get_installed_games(&self) -> QString {
        let games = omikuji_core::steam::get_installed_games();
        let json_games: Vec<serde_json::Value> = games.iter().map(|g| {
            serde_json::json!({
                "appid": g.appid,
                "name": g.name,
                "is_installed": g.is_installed()
            })
        }).collect();

        match serde_json::to_string(&json_games) {
            Ok(json) => QString::from(&json),
            Err(_) => QString::from("[]"),
        }
    }

    pub fn steam_local_library_image(&self, appid: &QString) -> QString {
        let appid_str = appid.to_string();
        match omikuji_core::steam::local::find_local_library_image(&appid_str) {
            Some(path) => QString::from(&*path.to_string_lossy()),
            None => QString::default(),
        }
    }

    pub fn steam_import_game(mut self: Pin<&mut Self>, appid: &QString, name: &QString) -> bool {
        let appid_str = appid.to_string();
        let name_str = name.to_string();

        eprintln!("[steam_import] importing {} - {}", appid_str, name_str);

        let already_imported = self.library.game.iter().any(|g| {
            g.metadata.id == appid_str
        });

        if already_imported {
            eprintln!("[steam_import] already imported: {}", appid_str);
            return true;
        }

        use omikuji_core::library::{Metadata, RunnerConfig, SourceConfig, WineConfig, LaunchConfig, GraphicsConfig, SystemConfig, default_color};

        let mut game = Game {
            metadata: Metadata {
                id: appid_str.clone(),
                name: name_str.clone(),
                sort_name: String::new(),
                slug: String::new(),
                exe: PathBuf::new(),
                color: default_color(),
                playtime: 0.0,
                last_played: String::new(),
                banner: String::new(),
                coverart: String::new(),
                icon: String::new(),
                favourite: false,
                categories: Vec::new(),
            },
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

        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("[steam_import] failed to save game: {}", e);
            return false;
        }

        let appid_for_media = appid_str.clone();
        std::thread::spawn(move || {
            let result = media::fetch_steam_media_blocking(&appid_for_media);
            let fetched: Vec<&str> = [
                result.banner.as_ref().map(|_| "banner"),
                result.coverart.as_ref().map(|_| "coverart"),
            ]
            .into_iter()
            .flatten()
            .collect();

            if fetched.is_empty() {
                eprintln!("no steam media found for appid {}", appid_for_media);
            } else {
                eprintln!("fetched steam {} for appid {}", fetched.join(", "), appid_for_media);
            }
        });

        self.as_mut()
            .begin_insert_rows(&QModelIndex::default(), row, row);

        self.as_mut()
            .rust_mut()
            .get_mut()
            .library
            .game
            .push(game);

        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        eprintln!("[steam_import] imported '{}' (steam appid: {})", name_str, appid_str);
        true
    }

    pub fn steam_sync_playtime(mut self: Pin<&mut Self>) {
        let api_key = omikuji_core::settings::get().steam.api_key.clone();
        if api_key.is_empty() {
            return;
        }

        eprintln!("[steam_sync] syncing playtime from steam api...");
        let qt_thread = self.as_mut().qt_thread();

        // blocking reqwest inside #[tokio::main] panics; escape to an os thread, then marshal the mutation back via qt_thread.queue
        std::thread::spawn(move || {
            let fetch_result = omikuji_core::steam::fetch_playtime_data(&api_key);

            let _ = qt_thread.queue(move |mut obj: Pin<&mut super::qobject::GameModel>| {
                let steam_data = match fetch_result {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[steam_sync] failed: {}", e);
                        return;
                    }
                };

                let library = &mut obj.as_mut().rust_mut().get_mut().library;
                let (updated, total) = omikuji_core::steam::apply_playtime_data(library, &steam_data);
                eprintln!("[steam_sync] updated {}/{} steam games", updated, total);

                let mut saved = 0;
                for game in &library.game {
                    if game.runner.runner_type == "steam" {
                        if let Err(e) = Library::save_game_static(game) {
                            eprintln!("[steam_sync] failed to save {}: {}", game.metadata.id, e);
                        } else {
                            saved += 1;
                        }
                    }
                }
                eprintln!("[steam_sync] saved {} games to disk", saved);

                obj.as_mut().begin_reset_model();
                obj.as_mut().end_reset_model();
            });
        });
    }
}
