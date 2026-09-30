use std::collections::HashSet;
use std::path::PathBuf;
use std::thread;

use crate::app_settings::AppSettings;
use crate::downloads::{self, DownloadKind, DownloadRequest};
use crate::gacha::strategies::{self, GachaUpdateInfo};
use crate::launch;
use crate::library::{Game, SourceKind};
use crate::media::{self, MediaType};
use crate::process::{UpdateKind, UpdateNotification};
use crate::store::{UpdateInfo, epic, gog, nile};

#[derive(Default)]
pub struct ScanCounts {
    pub epic: i32,
    pub gog: i32,
    pub nile: i32,
    pub gacha: i32,
}

impl ScanCounts {
    fn record(&mut self, kind: SourceKind) {
        match kind {
            SourceKind::Epic => self.epic += 1,
            SourceKind::Gog => self.gog += 1,
            SourceKind::Nile => self.nile += 1,
            SourceKind::Gacha => self.gacha += 1,
            SourceKind::Manual | SourceKind::Steam => {}
        }
    }
}

pub fn find_update(game: &Game) -> Option<UpdateNotification> {
    match game.source.kind {
        SourceKind::Gacha => {
            blocking_check_gacha_update(game).map(|info| gacha_notification(game, info))
        }
        _ => store_update(game).map(|info| store_notification(game, info)),
    }
}

pub fn pre_launch_check(game: &Game) -> Option<UpdateNotification> {
    if launch::precheck_exe(game).is_err() {
        return None;
    }
    let behavior = || AppSettings::load().behavior;
    match game.source.kind {
        SourceKind::Gacha => unmuted_gacha_update(game).map(|info| gacha_notification(game, info)),
        SourceKind::Epic if behavior().auto_check_epic_updates_on_launch => find_update(game),
        SourceKind::Gog if behavior().auto_check_gog_updates_on_launch => find_update(game),
        _ => None,
    }
}

pub fn boot_scan(games: &[Game]) -> Option<ScanCounts> {
    let behavior = AppSettings::load().behavior;
    if !behavior.auto_check_updates_on_boot {
        return None;
    }

    let candidates: Vec<&Game> = games.iter().filter(|g| is_scannable(g)).collect();
    candidates
        .iter()
        .map(|g| g.source.kind)
        .collect::<HashSet<_>>()
        .into_iter()
        .for_each(refresh_store_cache);

    let queued: HashSet<String> = downloads::manager()
        .list()
        .into_iter()
        .map(|e| e.app_id)
        .collect();

    let mut counts = ScanCounts::default();
    for game in candidates {
        if queued.contains(&game.source.app_id) {
            continue;
        }
        let Some(kind) = scan_update(game, behavior.auto_queue_predownloads_on_boot) else {
            continue;
        };
        let Some(req) = download_request(game, kind) else {
            continue;
        };
        downloads::manager().enqueue(DownloadRequest {
            start_paused: true,
            ..req
        });
        counts.record(game.source.kind);
    }
    Some(counts)
}

pub fn download_request(game: &Game, kind: DownloadKind) -> Option<DownloadRequest> {
    let (source, banner_url) = download_source(game)?;
    Some(DownloadRequest {
        source,
        app_id: game.source.app_id.clone(),
        game_id: game.metadata.id.clone(),
        display_name: format!("{} · {}", game.metadata.name, kind.label()),
        banner_url,
        install_path: strategies::game_install_root(game),
        prefix_path: (!game.wine.prefix.is_empty()).then(|| PathBuf::from(&game.wine.prefix)),
        runner_version: game.wine.version.clone(),
        temp_dir: None,
        kind,
        destructive_cleanup: false,
        start_paused: false,
        dlcs: game.source.dlcs.clone(),
        options: Vec::new(),
        packs: Vec::new(),
    })
}

fn download_source(game: &Game) -> Option<(String, Option<String>)> {
    match game.source.kind {
        SourceKind::Gacha => gacha_source(&game.source.app_id),
        SourceKind::Epic | SourceKind::Gog | SourceKind::Nile => {
            let banner =
                media::resolve_image(&game.metadata.id, &game.metadata.banner, &MediaType::Banner);
            Some((
                game.source.kind.as_str().to_string(),
                (!banner.is_empty()).then_some(banner),
            ))
        }
        kind => {
            tracing::error!(
                "unsupported source.kind '{}' for game '{}'",
                kind.as_str(),
                game.metadata.id
            );
            None
        }
    }
}

fn gacha_source(app_id: &str) -> Option<(String, Option<String>)> {
    let Some((manifest, edition_id)) = strategies::find_for_app_id(app_id) else {
        tracing::error!("no gacha manifest for app_id '{}'", app_id);
        return None;
    };
    let source = match strategies::source_key(&manifest, &edition_id) {
        Ok(s) => s.to_string(),
        Err(e) => {
            tracing::error!("unknown strategy for '{}': {}", manifest.id, e);
            return None;
        }
    };
    let poster = strategies::resolve_poster(&manifest);
    Some((source, (!poster.is_empty()).then_some(poster)))
}

fn is_scannable(game: &Game) -> bool {
    game.source.kind.has_updates()
        && !game.source.app_id.is_empty()
        && launch::validate_exe(game).is_ok()
}

fn scan_update(game: &Game, queue_predownloads: bool) -> Option<DownloadKind> {
    if game.source.kind == SourceKind::Gacha {
        let info = unmuted_gacha_update(game)?;
        return (queue_predownloads || info.kind != UpdateKind::PreDownload)
            .then(|| gacha_download_kind(info));
    }
    let info = cached_store_update(game)?;
    Some(DownloadKind::Update {
        from_version: info.from_version,
    })
}

fn gacha_download_kind(info: GachaUpdateInfo) -> DownloadKind {
    match info.kind {
        UpdateKind::PreDownload => DownloadKind::PreDownload {
            from_version: info.from_version,
            to_version: info.to_version,
        },
        UpdateKind::Required | UpdateKind::PreDownloaded => DownloadKind::Update {
            from_version: info.from_version,
        },
    }
}

fn refresh_store_cache(kind: SourceKind) {
    let refreshed = match kind {
        SourceKind::Epic => epic::updates::refresh_assets_cache(),
        SourceKind::Nile => nile::updates::refresh_updates_cache(),
        _ => return,
    };
    if let Err(e) = refreshed {
        tracing::warn!("{} update refresh failed: {:#}", kind.as_str(), e);
    }
}

fn store_update(game: &Game) -> Option<UpdateInfo> {
    refresh_store_cache(game.source.kind);
    cached_store_update(game)
}

fn cached_store_update(game: &Game) -> Option<UpdateInfo> {
    let app_id = &game.source.app_id;
    match game.source.kind {
        SourceKind::Epic => epic::updates::find_update_for(app_id),
        SourceKind::Gog => gog::updates::blocking_check_gog_update(app_id),
        SourceKind::Nile => nile::updates::find_update_for(app_id),
        _ => None,
    }
}

fn store_notification(game: &Game, info: UpdateInfo) -> UpdateNotification {
    UpdateNotification {
        game_id: game.metadata.id.clone(),
        app_id: game.source.app_id.clone(),
        from_version: info.from_version,
        to_version: info.to_version,
        download_size: 0,
        can_diff: true,
        delta_supported: true,
        kind: UpdateKind::Required,
    }
}

fn gacha_notification(game: &Game, info: GachaUpdateInfo) -> UpdateNotification {
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

fn unmuted_gacha_update(game: &Game) -> Option<GachaUpdateInfo> {
    blocking_check_gacha_update(game).filter(|info| !info.is_muted_for(game))
}

// callers can sit on a thread already inside the tokio runtime, and nesting a second one panics
fn blocking_check_gacha_update(game: &Game) -> Option<GachaUpdateInfo> {
    let game = game.clone();
    thread::spawn(move || {
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
