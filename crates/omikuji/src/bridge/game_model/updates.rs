use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use omikuji_core::app_settings::AppSettings;
use omikuji_core::defaults::Defaults;
use omikuji_core::downloads::{self, DownloadKind, DownloadRequest};
use omikuji_core::gacha::strategies::{self, GachaUpdateInfo};
use omikuji_core::library::{Game, Library, SourceKind};
use omikuji_core::media::{self, MediaType};
use omikuji_core::process::{self, UpdateKind, UpdateNotification};
use omikuji_core::store::{epic, gog, nile};
use omikuji_core::{launch, notifications, runners};

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
        if !matches!(
            game.source.kind,
            SourceKind::Gacha | SourceKind::Epic | SourceKind::Gog | SourceKind::Nile
        ) {
            return false;
        }
        if let Err(e) = launch::validate_exe(&game) {
            notifications::error(&game.metadata.name, e.to_string());
            return false;
        }

        notifications::info(&game.metadata.name, "Checking for updates...");

        std::thread::spawn(move || match find_update(&game) {
            Some(n) => process::notify_update_required(n),
            None => notifications::info(&game.metadata.name, "You're on the latest version"),
        });

        true
    }

    pub fn dismiss_predownload(mut self: Pin<&mut Self>, game_id: &QString, version: &QString) {
        let gid = game_id.to_string();
        let Some(idx) = self.library.game.iter().position(|g| g.metadata.id == gid) else {
            return;
        };
        let game = &mut self.as_mut().rust_mut().get_mut().library.game[idx];
        game.source.predownload_dismissed = version.to_string();
        if let Err(e) = Library::save_game_static(game) {
            tracing::error!("saving pre-download dismissal for '{}': {}", gid, e);
        }
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

    // TODO: prob use a core::updates cuz this is so bad man
    pub fn scan_all_for_updates(mut self: Pin<&mut Self>) {
        let settings = AppSettings::load();
        if !settings.behavior.auto_check_updates_on_boot {
            return;
        }
        let queue_predownloads = settings.behavior.auto_queue_predownloads_on_boot;

        struct ScanCandidate {
            source: SourceKind,
            app_id: String,
            game_id: String,
            display_name: String,
            banner_url: Option<String>,
            install_path: std::path::PathBuf,
            prefix_path: Option<std::path::PathBuf>,
            runner_version: String,
            dlcs: Vec<String>,
        }

        let candidates: Vec<ScanCandidate> = self
            .library
            .game
            .iter()
            .filter(|g| {
                matches!(
                    g.source.kind,
                    SourceKind::Epic | SourceKind::Gog | SourceKind::Nile
                ) && !g.source.app_id.is_empty()
                    && launch::validate_exe(g).is_ok()
            })
            .map(|g| {
                let install_path = std::path::PathBuf::from(&g.metadata.exe)
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from(&g.metadata.exe));
                let resolved =
                    media::resolve_image(&g.metadata.id, &g.metadata.banner, &MediaType::Banner);
                let banner_url = if resolved.is_empty() {
                    None
                } else {
                    Some(resolved)
                };
                let prefix_path = if g.wine.prefix.is_empty() {
                    None
                } else {
                    Some(std::path::PathBuf::from(&g.wine.prefix))
                };
                ScanCandidate {
                    source: g.source.kind,
                    app_id: g.source.app_id.clone(),
                    game_id: g.metadata.id.clone(),
                    display_name: g.metadata.name.clone(),
                    banner_url,
                    install_path,
                    prefix_path,
                    runner_version: g.wine.version.clone(),
                    dlcs: g.source.dlcs.clone(),
                }
            })
            .collect();

        let gacha_candidates: Vec<Game> = self
            .library
            .game
            .iter()
            .filter(|g| {
                g.source.kind == SourceKind::Gacha
                    && !g.source.app_id.is_empty()
                    && launch::validate_exe(g).is_ok()
            })
            .cloned()
            .collect();

        if candidates.is_empty() && gacha_candidates.is_empty() {
            return;
        }

        let sender = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            // epic batches one assets refresh up front, gog as no batch step
            if candidates.iter().any(|c| c.source == SourceKind::Epic) {
                let _ = epic::updates::refresh_assets_cache();
            }
            if candidates.iter().any(|c| c.source == SourceKind::Nile) {
                let _ = nile::updates::refresh_updates_cache();
            }

            let existing_app_ids: std::collections::HashSet<String> = downloads::manager()
                .list()
                .iter()
                .map(|e| e.app_id.clone())
                .collect();

            let mut epic_count: i32 = 0;
            let mut gog_count: i32 = 0;
            let mut nile_count: i32 = 0;

            for candidate in candidates {
                if existing_app_ids.contains(&candidate.app_id) {
                    continue;
                }
                let from_version = match candidate.source {
                    SourceKind::Epic => {
                        epic::updates::find_update_for(&candidate.app_id).map(|i| i.from_version)
                    }
                    SourceKind::Gog => gog::updates::blocking_check_gog_update(&candidate.app_id)
                        .map(|i| i.from_version),
                    SourceKind::Nile => {
                        nile::updates::find_update_for(&candidate.app_id).map(|i| i.from_version)
                    }
                    _ => None,
                };
                let Some(from_version) = from_version else {
                    continue;
                };

                let req = DownloadRequest {
                    source: candidate.source.as_str().to_string(),
                    app_id: candidate.app_id,
                    game_id: candidate.game_id,
                    display_name: format!("{} · update", candidate.display_name),
                    banner_url: candidate.banner_url,
                    install_path: candidate.install_path,
                    prefix_path: candidate.prefix_path,
                    runner_version: candidate.runner_version,
                    temp_dir: None,
                    kind: DownloadKind::Update { from_version },
                    destructive_cleanup: false,
                    start_paused: true,
                    dlcs: candidate.dlcs,
                    options: Vec::new(),
                };

                let _ = downloads::manager().enqueue(req);
                match candidate.source {
                    SourceKind::Epic => epic_count += 1,
                    SourceKind::Gog => gog_count += 1,
                    SourceKind::Nile => nile_count += 1,
                    _ => {}
                }
            }

            let mut gacha_count: i32 = 0;
            if !gacha_candidates.is_empty() {
                match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => {
                        for game in gacha_candidates {
                            if existing_app_ids.contains(&game.source.app_id) {
                                continue;
                            }
                            let Some(info) = rt.block_on(strategies::check_game_for_update(&game))
                            else {
                                continue;
                            };
                            if info.is_muted_for(&game) {
                                continue;
                            }
                            let (kind, label) = match info.kind {
                                UpdateKind::PreDownload if !queue_predownloads => continue,
                                UpdateKind::PreDownload => (
                                    DownloadKind::PreDownload {
                                        from_version: info.from_version,
                                        to_version: info.to_version,
                                    },
                                    PREDOWNLOAD_LABEL,
                                ),
                                UpdateKind::Required | UpdateKind::PreDownloaded => (
                                    DownloadKind::Update {
                                        from_version: info.from_version,
                                    },
                                    UPDATE_LABEL,
                                ),
                            };
                            let Some((source_key, banner_url)) =
                                resolve_gacha_source(&game.source.app_id)
                            else {
                                continue;
                            };
                            let mut req =
                                build_download_request(&game, source_key, banner_url, kind, label);
                            req.start_paused = true;
                            let _ = downloads::manager().enqueue(req);
                            gacha_count += 1;
                        }
                    }
                    Err(e) => tracing::error!("gacha update scan: runtime build failed: {}", e),
                }
            }

            let _ = sender.queue(move |mut m: Pin<&mut super::qobject::GameModel>| {
                m.as_mut()
                    .updates_queued(epic_count, gog_count, nile_count, gacha_count);
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
        self.enqueue_for_game(game_id, kind, UPDATE_LABEL)
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
        self.enqueue_for_game(game_id, kind, PREDOWNLOAD_LABEL)
    }

    fn enqueue_for_game(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        kind: DownloadKind,
        label: &str,
    ) -> QString {
        let gid = game_id.to_string();

        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            tracing::error!("enqueue_for_game: game '{}' not found", gid);
            return QString::from("");
        };
        if let Err(e) = launch::validate_exe(game) {
            tracing::warn!("enqueue_for_game: refusing '{}': {}", gid, e);
            return QString::from("");
        }

        let (source_key, banner_url) = if game.source.kind == SourceKind::Gacha {
            let Some(resolved) = resolve_gacha_source(&game.source.app_id) else {
                return QString::from("");
            };
            resolved
        } else if matches!(
            game.source.kind,
            SourceKind::Epic | SourceKind::Gog | SourceKind::Nile
        ) {
            let resolved =
                media::resolve_image(&game.metadata.id, &game.metadata.banner, &MediaType::Banner);
            (
                game.source.kind.as_str().to_string(),
                (!resolved.is_empty()).then_some(resolved),
            )
        } else {
            tracing::error!(
                "unsupported source.kind '{}' for game '{}'",
                game.source.kind.as_str(),
                gid
            );
            return QString::from("");
        };

        let req = build_download_request(game, source_key, banner_url, kind, label);

        let id = downloads::manager().enqueue(req);
        let _ = self.as_mut();
        QString::from(&id)
    }

    pub fn game_supports_repair(&self, game_id: &QString) -> bool {
        let gid = game_id.to_string();
        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            return false;
        };
        if game.source.kind != SourceKind::Gacha {
            return false;
        }
        let Some((manifest, edition_id, _)) = strategies::find_for_app_id(&game.source.app_id)
        else {
            return false;
        };
        match strategies::source_key(&manifest, &edition_id) {
            Ok(key) => downloads::manager().source_supports_repair(key),
            Err(_) => false,
        }
    }

    pub fn enqueue_game_repair(mut self: Pin<&mut Self>, game_id: &QString) -> QString {
        let gid = game_id.to_string();

        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            tracing::error!("enqueue_game_repair: game '{}' not found", gid);
            return QString::from("");
        };

        if game.source.kind != SourceKind::Gacha {
            tracing::error!("enqueue_game_repair: '{}' is not a gacha game", gid);
            return QString::from("");
        }

        let Some((source_key, banner_url)) = resolve_gacha_source(&game.source.app_id) else {
            return QString::from("");
        };

        let req =
            build_download_request(game, source_key, banner_url, DownloadKind::Repair, "repair");

        let id = downloads::manager().enqueue(req);
        let _ = self.as_mut();
        QString::from(&id)
    }
}

fn resolve_gacha_source(app_id: &str) -> Option<(String, Option<String>)> {
    let Some((manifest, edition_id, _voices)) = strategies::find_for_app_id(app_id) else {
        tracing::error!("no gacha manifest for app_id '{}'", app_id);
        return None;
    };
    let src = match strategies::source_key(&manifest, &edition_id) {
        Ok(s) => s.to_string(),
        Err(e) => {
            tracing::error!("unknown strategy for '{}': {}", manifest.id, e);
            return None;
        }
    };
    let poster = strategies::resolve_poster(&manifest);
    Some((
        src,
        if poster.is_empty() {
            None
        } else {
            Some(poster)
        },
    ))
}

fn build_download_request(
    game: &Game,
    source: String,
    banner_url: Option<String>,
    kind: DownloadKind,
    label: &str,
) -> DownloadRequest {
    let install_path = strategies::game_install_root(game);

    let prefix = if game.wine.prefix.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(&game.wine.prefix))
    };

    DownloadRequest {
        source,
        app_id: game.source.app_id.clone(),
        game_id: game.metadata.id.clone(),
        display_name: format!("{} · {}", game.metadata.name, label),
        banner_url,
        install_path,
        prefix_path: prefix,
        runner_version: game.wine.version.clone(),
        temp_dir: None,
        kind,
        // update/repair operate on an existing install, never wipe on cancel
        destructive_cleanup: false,
        start_paused: false,
        dlcs: game.source.dlcs.clone(),
        options: Vec::new(),
    }
}

const UPDATE_LABEL: &str = "update";
const PREDOWNLOAD_LABEL: &str = "pre-download";

fn find_update(game: &Game) -> Option<UpdateNotification> {
    match game.source.kind {
        SourceKind::Gacha => {
            blocking_check_gacha_update(game).map(|info| gacha_notification(game, info))
        }
        SourceKind::Epic => epic_update(game),
        SourceKind::Gog => gog_update(game),
        SourceKind::Nile => nile_update(game),
        _ => None,
    }
}

fn store_notification(
    game: &Game,
    from_version: String,
    to_version: String,
    download_size: u64,
) -> UpdateNotification {
    UpdateNotification {
        game_id: game.metadata.id.clone(),
        app_id: game.source.app_id.clone(),
        from_version,
        to_version,
        download_size,
        can_diff: true,
        delta_supported: true,
        kind: UpdateKind::Required,
    }
}

pub(super) fn gacha_notification(game: &Game, info: GachaUpdateInfo) -> UpdateNotification {
    UpdateNotification {
        game_id: game.metadata.id.clone(),
        app_id: game.source.app_id.clone(),
        from_version: info.from_version,
        to_version: info.to_version,
        download_size: info.download_size,
        can_diff: info.can_diff,
        delta_supported: info.delta_supported,
        kind: info.kind,
    }
}

pub(super) fn epic_update(game: &Game) -> Option<UpdateNotification> {
    let info = epic::updates::blocking_check_epic_update(&game.source.app_id)?;
    Some(store_notification(
        game,
        info.from_version,
        info.to_version,
        info.download_size,
    ))
}

pub(super) fn gog_update(game: &Game) -> Option<UpdateNotification> {
    let info = gog::updates::blocking_check_gog_update(&game.source.app_id)?;
    Some(store_notification(
        game,
        info.from_version,
        info.to_version,
        info.download_size,
    ))
}

fn nile_update(game: &Game) -> Option<UpdateNotification> {
    let info = nile::updates::blocking_check_nile_update(&game.source.app_id)?;
    Some(store_notification(
        game,
        info.from_version,
        String::new(),
        info.download_size,
    ))
}

// launch_game is called from the Qt event loop, which already runs inside the
// #[tokio::main] runtime. building a second runtime on that thread panics
// ("cannot start a runtime from within a runtime"). a plain os thread gives us a clean context to block_on from
pub(super) fn blocking_check_gacha_update(game: &Game) -> Option<GachaUpdateInfo> {
    let game = game.clone();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!("update check: runtime build failed: {}", e);
                return None;
            }
        };
        rt.block_on(strategies::check_game_for_update(&game))
    })
    .join()
    .ok()
    .flatten()
}
