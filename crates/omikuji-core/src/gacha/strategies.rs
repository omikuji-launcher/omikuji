use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::hoyo::{HoyoEdition, VoiceLocale};
use super::manifest::GachaManifest;
use crate::downloads::{DownloadKind, DownloadRequest};

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

#[derive(Debug, Clone, Default)]
pub struct ExistingInstallInfo {
    pub scratch_bytes: u64,
    pub segments: u32,
    pub has_install: bool,
    pub installed_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateCheck {
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
    pub can_diff: bool,
    pub delta_supported: bool,
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

/// app_id format: "{app_id_prefix}:{edition_id}" or "{app_id_prefix}:{edition_id}:{voices_csv}"
pub fn build_app_id(manifest: &GachaManifest, edition_id: &str, voices: &[String]) -> String {
    if voices.is_empty() {
        format!("{}:{}", manifest.app_id_prefix, edition_id)
    } else {
        format!(
            "{}:{}:{}",
            manifest.app_id_prefix,
            edition_id,
            voices.join(",")
        )
    }
}

pub fn find_for_app_id(app_id: &str) -> Option<(GachaManifest, String, Vec<String>)> {
    let parts: Vec<&str> = app_id.splitn(3, ':').collect();
    if parts.len() < 2 {
        return None;
    }
    let prefix = parts[0];
    let edition_id = parts[1].to_string();
    let voices: Vec<String> = parts
        .get(2)
        .map(|s| {
            s.split(',')
                .map(|v| v.trim())
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string())
                .collect()
        })
        .unwrap_or_default();
    let manifest = super::manifest::load_all()
        .into_iter()
        .find(|m| m.app_id_prefix == prefix)?;
    Some((manifest, edition_id, voices))
}

pub fn edition_exe_name<'a>(manifest: &'a GachaManifest, edition_id: &str) -> Option<&'a str> {
    manifest
        .edition(edition_id)
        .map(|e| e.exe_name.as_str())
        .filter(|s| !s.is_empty())
}

pub fn install_root_for(app_id: &str, exe: &Path) -> Option<PathBuf> {
    let (manifest, edition_id, _) = find_for_app_id(app_id)?;
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
    voices: &[String],
    install_path: PathBuf,
    prefix_path: Option<PathBuf>,
    runner_version: String,
    temp_dir: Option<PathBuf>,
) -> Result<DownloadRequest> {
    let edition = manifest.require_edition(edition_id)?;
    let source = manifest.strategy_for(edition).source_key().to_string();
    let app_id = build_app_id(manifest, edition_id, voices);
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
        alongside: false,
    })
}

pub fn detect_edition(manifest: &GachaManifest, install_path: &Path) -> Option<String> {
    manifest
        .editions
        .iter()
        .any(|e| manifest.strategy_for(e) == InstallStrategy::YostarFileIndex)
        .then(|| crate::gacha::yostar::detect_edition(manifest, install_path))
        .flatten()
}

pub fn supports_import(manifest: &GachaManifest, edition_id: &str) -> bool {
    source_key(manifest, edition_id)
        .is_ok_and(|k| crate::downloads::manager().source_supports_import(k))
}

pub async fn fetch_install_size(
    manifest: &GachaManifest,
    edition_id: &str,
    voices: &[String],
) -> Result<InstallSize> {
    match strategy(manifest, edition_id)? {
        InstallStrategy::HoyoSophon => {
            let edition = HoyoEdition::from_id(edition_id)?;
            let biz_id = crate::gacha::hoyo::biz_id(manifest, edition_id)?;
            let voice_locales: Vec<_> = voices
                .iter()
                .filter_map(|v| VoiceLocale::from_api_name(v))
                .collect();
            let s = crate::gacha::hoyo::api::fetch_install_size(&biz_id, edition, &voice_locales)
                .await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        InstallStrategy::GryphlineResourcePatch => {
            let s = crate::gacha::gryphline::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        InstallStrategy::KuroResourceIndex => {
            let s = crate::gacha::kuro::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        InstallStrategy::YostarFileIndex => {
            crate::gacha::yostar::api::fetch_install_size(manifest, edition_id).await
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
            let biz_id = crate::gacha::hoyo::biz_id(manifest, edition_id).ok()?;
            crate::gacha::hoyo::update::check_for_update(&biz_id, &manifest.game_slug, edition)
                .await
        }
        InstallStrategy::GryphlineResourcePatch => {
            crate::gacha::gryphline::update::check_for_update(manifest, edition_id).await
        }
        InstallStrategy::YostarFileIndex => {
            crate::gacha::yostar::update::check_for_update(manifest, edition_id).await
        }
        InstallStrategy::KuroResourceIndex => {
            crate::gacha::kuro::update::check_for_update(manifest, edition_id).await
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
    })
}

pub fn read_install_version(
    manifest: &GachaManifest,
    edition_id: &str,
    install_path: &Path,
) -> Option<String> {
    let edition = manifest.edition(edition_id)?;
    let data_folder = &edition.data_folder;
    match manifest.strategy_for(edition) {
        InstallStrategy::HoyoSophon => {
            crate::gacha::hoyo::read_install_version(install_path, data_folder)
        }
        InstallStrategy::GryphlineResourcePatch => {
            crate::gacha::gryphline::read_install_version(install_path, data_folder)
        }
        InstallStrategy::KuroResourceIndex => {
            crate::gacha::kuro::read_install_version(install_path, data_folder)
        }
        InstallStrategy::YostarFileIndex => {
            crate::gacha::yostar::read_install_version(install_path, data_folder)
        }
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
    let app_id = build_app_id(manifest, edition_id, &[]);
    let mut info = match strategy {
        InstallStrategy::HoyoSophon => {
            let (bytes, segments) =
                crate::gacha::hoyo::source::inspect_hoyo_temp(&app_id, install_path, temp_dir);
            let has_install = edition_exe_name(manifest, edition_id)
                .is_some_and(|exe| install_path.join(exe).exists());
            ExistingInstallInfo {
                scratch_bytes: bytes,
                segments,
                has_install,
                installed_version: None,
            }
        }
        InstallStrategy::GryphlineResourcePatch => {
            let (bytes, segments) = crate::gacha::gryphline::source::inspect_gryphline_temp(
                &app_id,
                install_path,
                temp_dir,
            );
            let has_install = install_path
                .join(edition_exe_name(manifest, edition_id).unwrap_or("Endfield.exe"))
                .exists();
            ExistingInstallInfo {
                scratch_bytes: bytes,
                segments,
                has_install,
                installed_version: None,
            }
        }
        InstallStrategy::KuroResourceIndex | InstallStrategy::YostarFileIndex => {
            let has_install = edition_exe_name(manifest, edition_id)
                .is_some_and(|exe| install_path.join(exe).exists());
            ExistingInstallInfo {
                scratch_bytes: 0,
                segments: 0,
                has_install,
                installed_version: None,
            }
        }
    };
    if info.has_install {
        info.installed_version = read_install_version(manifest, edition_id, install_path);
    }
    info
}

pub fn resolve_poster(manifest: &GachaManifest) -> String {
    crate::gacha::art::resolve_art(manifest, "grid")
}
