use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use omikuji_core::media::{self, MediaType};

impl super::qobject::GameModel {
    pub fn check_epic_update(&self, game_id: &QString) -> bool {
        let gid = game_id.to_string();
        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            return false;
        };
        if game.source.kind != "epic" {
            return false;
        }
        let id = game.metadata.id.clone();
        let name = game.metadata.name.clone();
        let app_id = game.source.app_id.clone();

        omikuji_core::notifications::info(&name, "Checking for updates...");

        std::thread::spawn(move || {
            match omikuji_core::epic::updates::blocking_check_epic_update(&app_id) {
                Some(info) => {
                    omikuji_core::process::notify_update_required(
                        omikuji_core::process::UpdateNotification {
                            game_id: id,
                            app_id,
                            from_version: info.from_version,
                            to_version: info.to_version,
                            download_size: info.download_size,
                            can_diff: true,
                            delta_supported: true,
                        },
                    );
                }
                None => {
                    omikuji_core::notifications::info(&name, "You're on the latest version");
                }
            }
        });

        true
    }

    pub fn check_gog_update(&self, game_id: &QString) -> bool {
        let gid = game_id.to_string();
        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            return false;
        };
        if game.source.kind != "gog" {
            return false;
        }
        let id = game.metadata.id.clone();
        let name = game.metadata.name.clone();
        let app_id = game.source.app_id.clone();

        omikuji_core::notifications::info(&name, "Checking for updates...");

        std::thread::spawn(move || {
            match omikuji_core::gog::updates::blocking_check_gog_update(&app_id) {
                Some(info) => {
                    omikuji_core::process::notify_update_required(
                        omikuji_core::process::UpdateNotification {
                            game_id: id,
                            app_id,
                            from_version: info.from_version,
                            to_version: info.to_version,
                            download_size: info.download_size,
                            can_diff: true,
                            delta_supported: true,
                        },
                    );
                }
                None => {
                    omikuji_core::notifications::info(&name, "You're on the latest version");
                }
            }
        });

        true
    }

    pub fn scan_all_for_updates(mut self: Pin<&mut Self>) {
        let settings = omikuji_core::ui_settings::UiSettings::load();
        if !settings.behavior.auto_check_updates_on_boot {
            return;
        }

        struct ScanCandidate {
            source: String,
            app_id: String,
            display_name: String,
            banner_url: Option<String>,
            install_path: std::path::PathBuf,
            prefix_path: Option<std::path::PathBuf>,
            runner_version: String,
        }

        let candidates: Vec<ScanCandidate> = self.library.game.iter()
            .filter(|g| (g.source.kind == "epic" || g.source.kind == "gog") && !g.source.app_id.is_empty())
            .map(|g| {
                let install_path = std::path::PathBuf::from(&g.metadata.exe)
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from(&g.metadata.exe));
                let resolved = media::resolve_image(&g.metadata.id, &g.metadata.banner, &MediaType::Banner);
                let banner_url = if resolved.is_empty() { None } else { Some(resolved) };
                let prefix_path = if g.wine.prefix.is_empty() {
                    None
                } else {
                    Some(std::path::PathBuf::from(&g.wine.prefix))
                };
                ScanCandidate {
                    source: g.source.kind.clone(),
                    app_id: g.source.app_id.clone(),
                    display_name: g.metadata.name.clone(),
                    banner_url,
                    install_path,
                    prefix_path,
                    runner_version: g.wine.version.clone(),
                }
            })
            .collect();

        if candidates.is_empty() {
            return;
        }

        let sender = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            // epic batches one assets refresh up front, gog as no batch step
            if candidates.iter().any(|c| c.source == "epic") {
                let _ = omikuji_core::epic::updates::refresh_assets_cache();
            }

            let existing_app_ids: std::collections::HashSet<String> = omikuji_core::downloads::manager()
                .list()
                .iter()
                .map(|e| e.app_id.clone())
                .collect();

            let mut epic_count: i32 = 0;
            let mut gog_count: i32 = 0;

            for candidate in candidates {
                if existing_app_ids.contains(&candidate.app_id) {
                    continue;
                }
                let from_version = match candidate.source.as_str() {
                    "epic" => omikuji_core::epic::updates::find_update_for(&candidate.app_id)
                        .map(|i| i.from_version),
                    "gog" => omikuji_core::gog::updates::blocking_check_gog_update(&candidate.app_id)
                        .map(|i| i.from_version),
                    _ => None,
                };
                let Some(from_version) = from_version else {
                    continue;
                };

                let req = omikuji_core::downloads::DownloadRequest {
                    source: candidate.source.clone(),
                    app_id: candidate.app_id,
                    display_name: format!("{} · update", candidate.display_name),
                    banner_url: candidate.banner_url,
                    install_path: candidate.install_path,
                    prefix_path: candidate.prefix_path,
                    runner_version: candidate.runner_version,
                    temp_dir: None,
                    kind: omikuji_core::downloads::DownloadKind::Update { from_version },
                    destructive_cleanup: false,
                    start_paused: true,
                };

                let _ = omikuji_core::downloads::manager().enqueue(req);
                match candidate.source.as_str() {
                    "epic" => epic_count += 1,
                    "gog" => gog_count += 1,
                    _ => {}
                }
            }

            let _ = sender.queue(move |mut m: Pin<&mut super::qobject::GameModel>| {
                m.as_mut().updates_queued(epic_count, gog_count);
            });
        });
    }

    pub fn enqueue_game_update(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        from_version: &QString,
    ) -> QString {
        let gid = game_id.to_string();
        let from = from_version.to_string();

        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            eprintln!("[update] enqueue_game_update: game '{}' not found", gid);
            return QString::from("");
        };

        let app_id = game.source.app_id.clone();
        let display_name = game.metadata.name.clone();

        let (source_key, banner_url) = if game.source.kind == "gacha" {
            match omikuji_core::gachas::strategies::find_for_app_id(&app_id) {
                Some((manifest, _edition_id, _voices)) => {
                    let src = match omikuji_core::gachas::strategies::source_key(&manifest) {
                        Ok(s) => s.to_string(),
                        Err(e) => {
                            eprintln!("[update] unknown strategy for '{}': {}", manifest.id, e);
                            return QString::from("");
                        }
                    };
                    let poster = omikuji_core::gachas::strategies::resolve_poster(&manifest);
                    (src, if poster.is_empty() { None } else { Some(poster) })
                }
                None => {
                    eprintln!("[update] no gacha manifest for app_id '{}'", app_id);
                    return QString::from("");
                }
            }
        } else if game.source.kind == "epic" {
            let resolved = media::resolve_image(&game.metadata.id, &game.metadata.banner, &MediaType::Banner);
            let banner = if resolved.is_empty() { None } else { Some(resolved) };
            ("epic".to_string(), banner)
        } else if game.source.kind == "gog" {
            let resolved = media::resolve_image(&game.metadata.id, &game.metadata.banner, &MediaType::Banner);
            let banner = if resolved.is_empty() { None } else { Some(resolved) };
            ("gog".to_string(), banner)
        } else {
            eprintln!("[update] unsupported source.kind '{}' for game '{}'", game.source.kind, gid);
            return QString::from("");
        };

        // exe's parent dir is the game-data root; sophon patcher and resource probes need that, not th exe itself
        let install_path = std::path::PathBuf::from(&game.metadata.exe)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from(&game.metadata.exe));

        let prefix = if game.wine.prefix.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(&game.wine.prefix))
        };
        let runner_version = game.wine.version.clone();

        let req = omikuji_core::downloads::DownloadRequest {
            source: source_key,
            app_id,
            display_name: format!("{} · update", display_name),
            banner_url,
            install_path,
            prefix_path: prefix,
            runner_version,
            temp_dir: None,
            kind: omikuji_core::downloads::DownloadKind::Update { from_version: from },
            // updates patch an existing install, never wipe on cancel
            destructive_cleanup: false,
            start_paused: false,
        };

        let id = omikuji_core::downloads::manager().enqueue(req);
        let _ = self.as_mut();
        QString::from(&id)
    }
}
