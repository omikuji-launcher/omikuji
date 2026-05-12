// strategy dispatch. each strategy is a compiled code path that the manifest names by string.
// v1 wraps the existing per-source modules; the translation to manifest-only calls can happen
// later without changing the bridge or ui surface.

use anyhow::{anyhow, bail, Result};
use std::path::{Path, PathBuf};

use super::manifest::GachaManifest;
use crate::downloads::{DownloadKind, DownloadRequest};

pub const HOYO_SOPHON: &str = "hoyo_sophon";
pub const GRYPHLINE_RESOURCE_PATCH: &str = "gryphline_resource_patch";
pub const KURO_RESOURCE_INDEX: &str = "kuro_resource_index";

#[derive(Debug, Clone, Copy, Default)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ExistingInstallInfo {
    // leftover bytes in the scratch dir (resume-able partial download)
    pub scratch_bytes: u64,
    pub segments: u32,
    pub has_install: bool,
    pub installed_version: Option<String>,
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

pub fn source_key(manifest: &GachaManifest) -> Result<&'static str> {
    match manifest.install_strategy.as_str() {
        HOYO_SOPHON => Ok("hoyo"),
        GRYPHLINE_RESOURCE_PATCH => Ok("endfield"),
        KURO_RESOURCE_INDEX => Ok("kuro"),
        other => bail!("unknown install_strategy: {}", other),
    }
}

/// app_id format: "{app_id_prefix}:{edition_id}" or "{app_id_prefix}:{edition_id}:{voices_csv}"
pub fn build_app_id(manifest: &GachaManifest, edition_id: &str, voices: &[String]) -> String {
    if voices.is_empty() {
        format!("{}:{}", manifest.app_id_prefix, edition_id)
    } else {
        format!("{}:{}:{}", manifest.app_id_prefix, edition_id, voices.join(","))
    }
}

/// given an app_id, find the manifest, edition id, and voice ids that produced it
/// used by launch hooks and bridge dispatchers.
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

#[allow(clippy::too_many_arguments)]
pub fn build_install_request(
    manifest: &GachaManifest,
    edition_id: &str,
    voices: &[String],
    display_name: String,
    install_path: PathBuf,
    prefix_path: Option<PathBuf>,
    runner_version: String,
    temp_dir: Option<PathBuf>,
) -> Result<DownloadRequest> {
    require_edition(manifest, edition_id)?;
    let source = source_key(manifest)?.to_string();
    let app_id = build_app_id(manifest, edition_id, voices);
    let banner_url = resolve_poster(manifest);
    Ok(DownloadRequest {
        source,
        app_id,
        display_name,
        banner_url: if banner_url.is_empty() { None } else { Some(banner_url) },
        install_path,
        prefix_path,
        runner_version,
        temp_dir,
        kind: DownloadKind::Install,
        destructive_cleanup: true,
        start_paused: false,
    })
}

pub fn build_update_request(
    manifest: &GachaManifest,
    edition_id: &str,
    from_version: String,
    display_name: String,
    install_path: PathBuf,
    prefix_path: Option<PathBuf>,
    runner_version: String,
) -> Result<DownloadRequest> {
    require_edition(manifest, edition_id)?;
    let source = source_key(manifest)?.to_string();
    let app_id = build_app_id(manifest, edition_id, &[]);
    let banner_url = resolve_poster(manifest);
    Ok(DownloadRequest {
        source,
        app_id,
        display_name,
        banner_url: if banner_url.is_empty() { None } else { Some(banner_url) },
        install_path,
        prefix_path,
        runner_version,
        temp_dir: None,
        kind: DownloadKind::Update { from_version },
        destructive_cleanup: false,
        start_paused: false,
    })
}

fn require_edition(manifest: &GachaManifest, edition_id: &str) -> Result<()> {
    if manifest.editions.iter().any(|e| e.id == edition_id) {
        Ok(())
    } else {
        Err(anyhow!(
            "edition '{}' not found in manifest '{}'",
            edition_id,
            manifest.id
        ))
    }
}

pub async fn fetch_install_size(
    manifest: &GachaManifest,
    edition_id: &str,
    voices: &[String],
) -> Result<InstallSize> {
    require_edition(manifest, edition_id)?;
    match manifest.install_strategy.as_str() {
        HOYO_SOPHON => {
            let edition = hoyo_edition_from_id(edition_id)?;
            let biz_id = hoyo_biz_id(manifest, edition_id)?;
            let voice_locales = hoyo_voices_from_ids(voices);
            let s = crate::hoyo::api::fetch_install_size(&biz_id, edition, &voice_locales).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        GRYPHLINE_RESOURCE_PATCH => {
            let s = crate::endfield::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        KURO_RESOURCE_INDEX => {
            let s = crate::kuro::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        other => bail!("unknown install_strategy: {}", other),
    }
}

pub async fn check_for_update(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Option<GachaUpdateInfo> {
    require_edition(manifest, edition_id).ok()?;
    match manifest.install_strategy.as_str() {
        HOYO_SOPHON => {
            let edition = hoyo_edition_from_id(edition_id).ok()?;
            let biz_id = hoyo_biz_id(manifest, edition_id).ok()?;
            let info = crate::hoyo::update::check_for_update(&biz_id, &manifest.game_slug, edition).await.ok()??;
            Some(GachaUpdateInfo {
                manifest_id: manifest.id.clone(),
                edition_id: edition_id.to_string(),
                from_version: info.from_version,
                to_version: info.to_version,
                download_size: info.download_size,
                can_diff: info.can_diff,
                delta_supported: info.delta_supported,
            })
        }
        GRYPHLINE_RESOURCE_PATCH => {
            let info = crate::endfield::update::check_for_update(manifest, edition_id).await.ok()??;
            Some(GachaUpdateInfo {
                manifest_id: manifest.id.clone(),
                edition_id: edition_id.to_string(),
                from_version: info.from_version,
                to_version: info.to_version,
                download_size: info.download_size,
                can_diff: info.can_diff,
                delta_supported: info.delta_supported,
            })
        }
        KURO_RESOURCE_INDEX => {
            let info = crate::kuro::update::check_for_update(manifest, edition_id)
                .await
                .ok()??;
            Some(GachaUpdateInfo {
                manifest_id: manifest.id.clone(),
                edition_id: edition_id.to_string(),
                from_version: info.from_version,
                to_version: info.to_version,
                download_size: info.download_size,
                can_diff: info.can_diff,
                delta_supported: info.delta_supported,
            })
        }
        _ => None,
    }
}

pub fn installed_version(manifest: &GachaManifest, edition_id: &str) -> Option<String> {
    match manifest.install_strategy.as_str() {
        HOYO_SOPHON => {
            let edition = hoyo_edition_from_id(edition_id).ok()?;
            crate::hoyo::installed_version(&manifest.game_slug, edition)
        }
        GRYPHLINE_RESOURCE_PATCH => {
            crate::endfield::installed_version(&manifest.game_slug, edition_id)
        }
        KURO_RESOURCE_INDEX => crate::kuro::installed_version(&manifest.game_slug, edition_id),
        _ => None,
    }
}

pub fn read_install_version(
    manifest: &GachaManifest,
    edition_id: &str,
    install_path: &Path,
) -> Option<String> {
    let edition = manifest.editions.iter().find(|e| e.id == edition_id)?;
    match manifest.install_strategy.as_str() {
        HOYO_SOPHON => crate::hoyo::read_install_version(install_path, &edition.data_folder),
        GRYPHLINE_RESOURCE_PATCH => {
            crate::endfield::read_install_version(install_path, &edition.data_folder)
        }
        KURO_RESOURCE_INDEX => crate::kuro::read_install_version(install_path, &edition.data_folder),
        _ => None,
    }
}

pub fn inspect_existing(
    manifest: &GachaManifest,
    edition_id: &str,
    install_path: &Path,
    temp_dir: Option<&Path>,
) -> ExistingInstallInfo {
    let app_id = build_app_id(manifest, edition_id, &[]);
    let mut info = match manifest.install_strategy.as_str() {
        HOYO_SOPHON => {
            let (bytes, segments) =
                crate::hoyo::source::inspect_hoyo_temp(&app_id, install_path, temp_dir);
            // hoyo has no cheap "is installed" signal without touching the game's own manifest,
            // so fall back to checking whether the edition's exe exists
            let edition_exe = manifest
                .editions
                .iter()
                .find(|e| e.id == edition_id)
                .map(|e| e.exe_name.as_str())
                .unwrap_or("");
            let has_install = !edition_exe.is_empty() && install_path.join(edition_exe).exists();
            ExistingInstallInfo {
                scratch_bytes: bytes,
                segments,
                has_install,
                installed_version: None,
            }
        }
        GRYPHLINE_RESOURCE_PATCH => {
            let (bytes, segments) =
                crate::endfield::source::inspect_endfield_temp(&app_id, install_path, temp_dir);
            let edition_exe = manifest
                .editions
                .iter()
                .find(|e| e.id == edition_id)
                .map(|e| e.exe_name.as_str())
                .unwrap_or("Endfield.exe");
            let has_install = install_path.join(edition_exe).exists();
            ExistingInstallInfo {
                scratch_bytes: bytes,
                segments,
                has_install,
                installed_version: None,
            }
        }
        KURO_RESOURCE_INDEX => {
            // kuro writes files straight into install_path with no scratch dir.
            // partial downloads are invisible here; the resume hint wont light up
            // until source.install() runs and size-matches per-file.
            let edition_exe = manifest
                .editions
                .iter()
                .find(|e| e.id == edition_id)
                .map(|e| e.exe_name.as_str())
                .unwrap_or("");
            let has_install = !edition_exe.is_empty() && install_path.join(edition_exe).exists();
            ExistingInstallInfo {
                scratch_bytes: 0,
                segments: 0,
                has_install,
                installed_version: None,
            }
        }
        _ => ExistingInstallInfo::default(),
    };
    if info.has_install {
        info.installed_version = read_install_version(manifest, edition_id, install_path);
    }
    info
}

pub fn resolve_poster(manifest: &GachaManifest) -> String {
    crate::gachas::art::resolve_art(manifest, "grid")
}

fn hoyo_edition_from_id(id: &str) -> Result<crate::hoyo::HoyoEdition> {
    use crate::hoyo::HoyoEdition;
    match id {
        "global" => Ok(HoyoEdition::Global),
        "china" => Ok(HoyoEdition::China),
        other => bail!("no hoyo edition for id: {}", other),
    }
}

fn hoyo_voices_from_ids(ids: &[String]) -> Vec<crate::hoyo::VoiceLocale> {
    use crate::hoyo::VoiceLocale;
    ids.iter()
        .filter_map(|id| VoiceLocale::all().iter().find(|v| v.api_name() == id).copied())
        .collect()
}

fn hoyo_biz_id(manifest: &GachaManifest, edition_id: &str) -> Result<String> {
    manifest
        .editions
        .iter()
        .find(|e| e.id == edition_id)
        .and_then(|e| e.strategy_config.get("biz_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("no biz_id in manifest {} for edition {}", manifest.id, edition_id))
}
