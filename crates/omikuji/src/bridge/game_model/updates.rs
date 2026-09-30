use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use omikuji_core::defaults::Defaults;
use omikuji_core::downloads::{self, DownloadKind};
use omikuji_core::gacha::strategies;
use omikuji_core::library::{Game, SourceKind};
use omikuji_core::{launch, notifications, process, runners, updates};

impl super::qobject::GameModel {
    pub fn check_game_update(&self, game_id: &QString) -> bool {
        let gid = game_id.to_string();
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == gid)
            .cloned()
        else {
            return false;
        };
        if !game.source.kind.has_updates() {
            return false;
        }
        if let Err(e) = launch::validate_exe(&game) {
            notifications::error(&game.metadata.name, e.to_string());
            return false;
        }

        notifications::info(&game.metadata.name, "Checking for updates...");

        std::thread::spawn(move || match updates::find_update(&game) {
            Some(n) => process::notify_update_required(n),
            None => notifications::info(&game.metadata.name, "You're on the latest version"),
        });

        true
    }

    pub fn dismiss_predownload(self: Pin<&mut Self>, game_id: &QString, version: &QString) {
        let version = version.to_string();
        self.update_game(&game_id.to_string(), |g| {
            g.source.predownload_dismissed = version
        });
    }

    pub fn refresh_latest_runners(mut self: Pin<&mut Self>) {
        let selections: Vec<String> = self
            .library
            .game
            .iter()
            .filter(|g| g.uses_wine_prefix())
            .map(|g| g.wine.version.clone())
            .chain(Defaults::load().wine.version)
            .collect();

        let sources = runners::latest_sources_for(&selections);
        if sources.is_empty() {
            return;
        }

        let sender = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("latest runner refresh: runtime build failed: {}", e);
                    return;
                }
            };

            let mut pending = Vec::new();
            for source in sources {
                match rt.block_on(runners::latest_update(&source)) {
                    Ok(Some(release)) => pending.push((source, release)),
                    Ok(None) => {}
                    Err(e) => tracing::warn!(
                        "couldn't check {}: {}",
                        runners::latest_dir_name(&source),
                        e
                    ),
                }
            }

            if pending.is_empty() {
                return;
            }

            let names: Vec<&str> = pending.iter().map(|(s, _)| s.name.as_str()).collect();
            if let Ok(json) = serde_json::to_string(&names) {
                let _ = sender.queue(move |mut m: Pin<&mut super::qobject::GameModel>| {
                    m.as_mut().latest_refresh_queued(&QString::from(&json));
                });
            }

            for (source, release) in pending {
                if let Err(e) = rt.block_on(runners::install_latest_release(&source, &release)) {
                    tracing::warn!(
                        "couldn't refresh {}: {}",
                        runners::latest_dir_name(&source),
                        e
                    );
                }
                let name = source.name.clone();
                let _ = sender.queue(move |mut m: Pin<&mut super::qobject::GameModel>| {
                    m.as_mut().latest_refresh_done(&QString::from(&name));
                });
            }
        });
    }

    pub fn scan_all_for_updates(mut self: Pin<&mut Self>) {
        let games = self.library.game.clone();
        let sender = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let Some(counts) = updates::boot_scan(&games) else {
                return;
            };
            let _ = sender.queue(move |mut m: Pin<&mut super::qobject::GameModel>| {
                m.as_mut()
                    .updates_queued(counts.epic, counts.gog, counts.nile, counts.gacha);
            });
        });
    }

    pub fn enqueue_game_update(
        self: Pin<&mut Self>,
        game_id: &QString,
        from_version: &QString,
    ) -> QString {
        let kind = DownloadKind::Update {
            from_version: from_version.to_string(),
        };
        self.enqueue_for_game(game_id, kind)
    }

    pub fn enqueue_game_predownload(
        self: Pin<&mut Self>,
        game_id: &QString,
        from_version: &QString,
        to_version: &QString,
    ) -> QString {
        let kind = DownloadKind::PreDownload {
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
        };
        self.enqueue_for_game(game_id, kind)
    }

    fn enqueue_for_game(&self, game_id: &QString, kind: DownloadKind) -> QString {
        let gid = game_id.to_string();

        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            tracing::error!("enqueue_for_game: game '{}' not found", gid);
            return QString::from("");
        };
        if let Err(e) = launch::validate_exe(game) {
            tracing::warn!("enqueue_for_game: refusing '{}': {}", gid, e);
            return QString::from("");
        }

        enqueue(game, kind)
    }

    pub fn game_supports_repair(&self, game_id: &QString) -> bool {
        let gid = game_id.to_string();
        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            return false;
        };
        if game.source.kind != SourceKind::Gacha {
            return false;
        }
        let Some((manifest, edition_id)) = strategies::find_for_app_id(&game.source.app_id) else {
            return false;
        };
        match strategies::source_key(&manifest, &edition_id) {
            Ok(key) => downloads::manager().source_supports_repair(key),
            Err(_) => false,
        }
    }

    pub fn enqueue_game_repair(self: Pin<&mut Self>, game_id: &QString) -> QString {
        let gid = game_id.to_string();

        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            tracing::error!("enqueue_game_repair: game '{}' not found", gid);
            return QString::from("");
        };

        if game.source.kind != SourceKind::Gacha {
            tracing::error!("enqueue_game_repair: '{}' is not a gacha game", gid);
            return QString::from("");
        }

        enqueue(game, DownloadKind::Repair)
    }
}

fn enqueue(game: &Game, kind: DownloadKind) -> QString {
    updates::download_request(game, kind)
        .map(|req| QString::from(&downloads::manager().enqueue(req)))
        .unwrap_or_default()
}
