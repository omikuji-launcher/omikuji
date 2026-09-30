use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::hoyo::{self, HoyoEdition};
use super::manifest::GachaManifest;
use super::{art, gryphline, kuro, yostar};
use crate::downloads::{self, DownloadKind, DownloadRequest};
use crate::library::Game;
use crate::process::UpdateKind;
use crate::updates;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStrategy {
    HoyoSophon,
    GryphlineResourcePatch,
    KuroResourceIndex,
    YostarFileIndex,
}

impl InstallStrategy {
    pub fn source_key(self) -> &'static str {
        match self {
            Self::HoyoSophon => "hoyo",
            Self::GryphlineResourcePatch => "endfield",
            Self::KuroResourceIndex => "kuro",
            Self::YostarFileIndex => "yostar",
        }
    }

    pub fn needs_hpatchz(self) -> bool {
        matches!(self, Self::HoyoSophon | Self::GryphlineResourcePatch)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackInfo {
    pub id: String,
    pub label: String,
    pub installed: bool,
    pub downloading: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ExistingInstallInfo {
    pub scratch_bytes: u64,
    pub segments: u32,
    pub has_install: bool,
    pub installed_version: Option<String>,
    pub installed_packs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCheck {
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
    pub can_diff: bool,
    pub delta_supported: bool,
    pub kind: UpdateKind,
}

#[derive(Debug, Clone)]
pub struct GachaUpdateInfo {
    pub manifest_id: String,
    pub edition_id: String,
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
    pub can_diff: bool,
    pub delta_supported: bool,
    pub kind: UpdateKind,
}

impl GachaUpdateInfo {
    pub fn is_muted_for(&self, game: &Game) -> bool {
        self.kind == UpdateKind::PreDownload
            && (game.source.predownload_dismissed == self.to_version
                || downloads::manager().has_active_for_game(&game.metadata.id))
    }
}

pub fn normalize_version(v: &str) -> String {
    let mut parts: Vec<&str> = v.split('.').collect();
    while parts.len() > 1 && parts.last() == Some(&"0") {
        parts.pop();
    }
    parts.join(".")
}

pub fn strategy(manifest: &GachaManifest, edition_id: &str) -> Result<InstallStrategy> {
    manifest
        .require_edition(edition_id)
        .map(|e| manifest.strategy_for(e))
}

pub fn source_key(manifest: &GachaManifest, edition_id: &str) -> Result<&'static str> {
    strategy(manifest, edition_id).map(InstallStrategy::source_key)
}

/// app_id format: "{app_id_prefix}:{edition_id}"
pub fn build_app_id(manifest: &GachaManifest, edition_id: &str) -> String {
    format!("{}:{}", manifest.app_id_prefix, edition_id)
}

// older installs carry a third ":voices" segment that split just ignores
pub fn find_for_app_id(app_id: &str) -> Option<(GachaManifest, String)> {
    let mut parts = app_id.split(':');
    let prefix = parts.next()?;
    let edition_id = parts.next()?.to_string();
    let manifest = super::manifest::load_all()
        .into_iter()
        .find(|m| m.app_id_prefix == prefix)?;
    Some((manifest, edition_id))
}

pub fn edition_exe_name<'a>(manifest: &'a GachaManifest, edition_id: &str) -> Option<&'a str> {
    manifest
        .edition(edition_id)
        .map(|e| e.exe_name.as_str())
        .filter(|s| !s.is_empty())
}

pub fn install_root_for(app_id: &str, exe: &Path) -> Option<PathBuf> {
    let (manifest, edition_id) = find_for_app_id(app_id)?;
    let rel = Path::new(edition_exe_name(&manifest, &edition_id)?);
    if !exe.ends_with(rel) {
        return None;
    }
    exe.ancestors()
        .nth(rel.components().count())
        .map(Path::to_path_buf)
}

pub fn build_install_request(
    manifest: &GachaManifest,
    edition_id: &str,
    packs: Vec<String>,
    install_path: PathBuf,
    prefix_path: Option<PathBuf>,
    runner_version: String,
    temp_dir: Option<PathBuf>,
) -> Result<DownloadRequest> {
    let edition = manifest.require_edition(edition_id)?;
    let source = manifest.strategy_for(edition).source_key().to_string();
    let app_id = build_app_id(manifest, edition_id);
    let banner_url = resolve_poster(manifest);
    Ok(DownloadRequest {
        source,
        app_id,
        game_id: String::new(),
        display_name: manifest.display_name_for(edition),
        banner_url: if banner_url.is_empty() {
            None
        } else {
            Some(banner_url)
        },
        install_path,
        prefix_path,
        runner_version,
        temp_dir,
        kind: DownloadKind::Install,
        destructive_cleanup: true,
        start_paused: false,
        dlcs: Vec::new(),
        options: Vec::new(),
        packs,
    })
}

pub fn detect_edition(manifest: &GachaManifest, install_path: &Path) -> Option<String> {
    manifest
        .editions
        .iter()
        .any(|e| manifest.strategy_for(e) == InstallStrategy::YostarFileIndex)
        .then(|| yostar::detect_edition(manifest, install_path))
        .flatten()
}

pub fn supports_import(manifest: &GachaManifest, edition_id: &str) -> bool {
    source_key(manifest, edition_id).is_ok_and(|k| downloads::manager().source_supports_import(k))
}

pub async fn fetch_install_size(
    manifest: &GachaManifest,
    edition_id: &str,
    voices: &[String],
) -> Result<InstallSize> {
    match strategy(manifest, edition_id)? {
        InstallStrategy::HoyoSophon => {
            let edition = HoyoEdition::from_id(edition_id)?;
            let biz_id = hoyo::biz_id(manifest, edition_id)?;
            let s = hoyo::api::fetch_install_size(&biz_id, edition, voices).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        InstallStrategy::GryphlineResourcePatch => {
            let s = gryphline::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        InstallStrategy::KuroResourceIndex => {
            let s = kuro::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        InstallStrategy::YostarFileIndex => {
            yostar::api::fetch_install_size(manifest, edition_id).await
        }
    }
}

pub async fn check_for_update(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Option<GachaUpdateInfo> {
    let check = match strategy(manifest, edition_id).ok()? {
        InstallStrategy::HoyoSophon => {
            let edition = HoyoEdition::from_id(edition_id).ok()?;
            let biz_id = hoyo::biz_id(manifest, edition_id).ok()?;
            hoyo::update::check_for_update(&biz_id, &manifest.game_slug, edition).await
        }
        InstallStrategy::GryphlineResourcePatch => {
            gryphline::update::check_for_update(manifest, edition_id).await
        }
        InstallStrategy::YostarFileIndex => {
            yostar::update::check_for_update(manifest, edition_id).await
        }
        InstallStrategy::KuroResourceIndex => {
            kuro::update::check_for_update(manifest, edition_id).await
        }
    }
    .ok()??;
    Some(GachaUpdateInfo {
        manifest_id: manifest.id.clone(),
        edition_id: edition_id.to_string(),
        from_version: check.from_version,
        to_version: check.to_version,
        download_size: check.download_size,
        can_diff: check.can_diff,
        delta_supported: check.delta_supported,
        kind: check.kind,
    })
}

pub async fn check_game_for_update(game: &Game) -> Option<GachaUpdateInfo> {
    let (manifest, edition_id) = find_for_app_id(&game.source.app_id)?;
    let mut info = check_for_update(&manifest, &edition_id).await?;
    let staged = staged_version(&manifest, &edition_id, game);
    if staged.as_deref() == Some(info.to_version.as_str()) {
        match info.kind {
            UpdateKind::PreDownload => return None,
            UpdateKind::Required => info.kind = UpdateKind::PreDownloaded,
            UpdateKind::PreDownloaded => {}
        }
    }
    Some(info)
}

fn staged_version(manifest: &GachaManifest, edition_id: &str, game: &Game) -> Option<String> {
    match strategy(manifest, edition_id).ok()? {
        InstallStrategy::HoyoSophon => {
            hoyo::source::predownloaded_version(&game.source.app_id, &game_install_root(game))
        }
        _ => None,
    }
}

pub fn game_install_root(game: &Game) -> PathBuf {
    let exe = &game.metadata.exe;
    install_root_for(&game.source.app_id, exe)
        .or_else(|| exe.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| exe.clone())
}

pub fn packs(game: &Game) -> Vec<PackInfo> {
    let Some((manifest, edition_id)) = find_for_app_id(&game.source.app_id) else {
        return Vec::new();
    };
    let Some(edition) = manifest.edition(&edition_id) else {
        return Vec::new();
    };
    if manifest.strategy_for(edition) != InstallStrategy::HoyoSophon {
        return Vec::new();
    }
    let root = game_install_root(game);
    // sophon writes straight into the game dir so a queued pack's folder exists long before it's whole
    let in_queue = downloads::manager().packs_in_queue(&game.metadata.id);
    manifest
        .voice_locales
        .iter()
        .map(|voice| {
            let queued = in_queue.contains(&voice.id);
            PackInfo {
                id: voice.id.clone(),
                label: voice.label.clone(),
                installed: !queued && hoyo::voice_installed(&manifest, edition, voice, &root),
                downloading: queued,
            }
        })
        .collect()
}

// cancelling a pack job deletes that pack's files so it must never be queued over one that's already there
pub fn queue_pack(game: &Game, pack: &str) -> Result<String> {
    if packs(game).iter().any(|p| p.id == pack && p.installed) {
        bail!("{pack} is already installed");
    }
    let kind = DownloadKind::AddPack {
        pack: pack.to_string(),
    };
    let req = updates::download_request(game, kind)
        .ok_or_else(|| anyhow!("{} can't download packs", game.metadata.name))?;
    Ok(downloads::manager().enqueue(req))
}

pub fn remove_pack(game: &Game, pack: &str) -> Result<()> {
    if downloads::manager().has_active_for_game(&game.metadata.id) {
        bail!(
            "wait for {}'s downloads to finish first",
            game.metadata.name
        );
    }
    let (manifest, edition_id) = find_for_app_id(&game.source.app_id)
        .ok_or_else(|| anyhow!("no manifest for {}", game.source.app_id))?;
    let edition = manifest.require_edition(&edition_id)?;
    match manifest.strategy_for(edition) {
        InstallStrategy::HoyoSophon => {
            hoyo::remove_voice_pack(&manifest, edition, &game_install_root(game), pack)
        }
        _ => bail!("{} has no removable packs", game.metadata.name),
    }
}

pub fn read_install_version(
    manifest: &GachaManifest,
    edition_id: &str,
    install_path: &Path,
) -> Option<String> {
    let edition = manifest.edition(edition_id)?;
    let data_folder = &edition.data_folder;
    match manifest.strategy_for(edition) {
        InstallStrategy::HoyoSophon => hoyo::read_install_version(install_path, data_folder),
        InstallStrategy::GryphlineResourcePatch => {
            gryphline::read_install_version(install_path, data_folder)
        }
        InstallStrategy::KuroResourceIndex => kuro::read_install_version(install_path, data_folder),
        InstallStrategy::YostarFileIndex => yostar::read_install_version(install_path, data_folder),
    }
}

pub fn inspect_existing(
    manifest: &GachaManifest,
    edition_id: &str,
    install_path: &Path,
    temp_dir: Option<&Path>,
) -> ExistingInstallInfo {
    let Ok(strategy) = strategy(manifest, edition_id) else {
        return ExistingInstallInfo::default();
    };
    let app_id = build_app_id(manifest, edition_id);
    let mut info = match strategy {
        InstallStrategy::HoyoSophon => {
            let (bytes, segments) =
                hoyo::source::inspect_hoyo_temp(&app_id, install_path, temp_dir);
            let has_install = edition_exe_name(manifest, edition_id)
                .is_some_and(|exe| install_path.join(exe).exists());
            ExistingInstallInfo {
                scratch_bytes: bytes,
                segments,
                has_install,
                ..Default::default()
            }
        }
        InstallStrategy::GryphlineResourcePatch => {
            let (bytes, segments) =
                gryphline::source::inspect_gryphline_temp(&app_id, install_path, temp_dir);
            let has_install = install_path
                .join(edition_exe_name(manifest, edition_id).unwrap_or("Endfield.exe"))
                .exists();
            ExistingInstallInfo {
                scratch_bytes: bytes,
                segments,
                has_install,
                ..Default::default()
            }
        }
        InstallStrategy::KuroResourceIndex | InstallStrategy::YostarFileIndex => {
            let has_install = edition_exe_name(manifest, edition_id)
                .is_some_and(|exe| install_path.join(exe).exists());
            ExistingInstallInfo {
                has_install,
                ..Default::default()
            }
        }
    };
    if info.has_install {
        info.installed_version = read_install_version(manifest, edition_id, install_path);
        if strategy == InstallStrategy::HoyoSophon
            && let Some(edition) = manifest.edition(edition_id)
        {
            info.installed_packs = hoyo::installed_voice_ids(manifest, edition, install_path);
        }
    }
    info
}

pub fn resolve_poster(manifest: &GachaManifest) -> String {
    art::resolve_art(manifest, "grid")
}
