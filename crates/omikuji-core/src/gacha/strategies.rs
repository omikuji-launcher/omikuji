use anyhow::{Result, anyhow, bail};
use std::path::{Path, PathBuf};

use super::manifest::GachaManifest;
use crate::downloads::{DownloadKind, DownloadRequest};

pub const HOYO_SOPHON: &str = "hoyo_sophon";
pub const GRYPHLINE_RESOURCE_PATCH: &str = "gryphline_resource_patch";
pub const KURO_RESOURCE_INDEX: &str = "kuro_resource_index";
pub const YOSTAR_FILE_INDEX: &str = "yostar_file_index";

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
        YOSTAR_FILE_INDEX => Ok("yostar"),
        other => bail!("unknown install_strategy: {}", other),
    }
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
        .editions
        .iter()
        .find(|e| e.id == edition_id)
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
        game_id: String::new(),
        display_name,
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
    })
}

pub fn detect_edition(manifest: &GachaManifest, install_path: &Path) -> Option<String> {
    match manifest.install_strategy.as_str() {
        YOSTAR_FILE_INDEX => crate::gacha::yostar::detect_edition(manifest, install_path),
        _ => None,
    }
}

pub fn supports_import(manifest: &GachaManifest) -> bool {
    source_key(manifest)
        .map(|k| crate::downloads::manager().source_supports_import(k))
        .unwrap_or(false)
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
        game_id: String::new(),
        display_name,
        banner_url: if banner_url.is_empty() {
            None
        } else {
            Some(banner_url)
        },
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
            let s = crate::gacha::hoyo::api::fetch_install_size(&biz_id, edition, &voice_locales)
                .await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        GRYPHLINE_RESOURCE_PATCH => {
            let s = crate::gacha::gryphline::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        KURO_RESOURCE_INDEX => {
            let s = crate::gacha::kuro::api::fetch_install_size(manifest, edition_id).await?;
            Ok(InstallSize {
                download_bytes: s.download_bytes,
                install_bytes: s.install_bytes,
            })
        }
        YOSTAR_FILE_INDEX => {
            crate::gacha::yostar::api::fetch_install_size(manifest, edition_id).await
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
            let info =
                crate::gacha::hoyo::update::check_for_update(&biz_id, &manifest.game_slug, edition)
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
        GRYPHLINE_RESOURCE_PATCH => {
            let info = crate::gacha::gryphline::update::check_for_update(manifest, edition_id)
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
        YOSTAR_FILE_INDEX => {
            let info = crate::gacha::yostar::update::check_for_update(manifest, edition_id)
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
        KURO_RESOURCE_INDEX => {
            let info = crate::gacha::kuro::update::check_for_update(manifest, edition_id)
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
            crate::gacha::hoyo::installed_version(&manifest.game_slug, edition)
        }
        GRYPHLINE_RESOURCE_PATCH => {
            crate::gacha::gryphline::installed_version(&manifest.game_slug, edition_id)
        }
        KURO_RESOURCE_INDEX => {
            crate::gacha::kuro::installed_version(&manifest.game_slug, edition_id)
        }
        YOSTAR_FILE_INDEX => {
            crate::gacha::yostar::installed_version(&manifest.game_slug, edition_id)
        }
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
        HOYO_SOPHON => crate::gacha::hoyo::read_install_version(install_path, &edition.data_folder),
        GRYPHLINE_RESOURCE_PATCH => {
            crate::gacha::gryphline::read_install_version(install_path, &edition.data_folder)
        }
        KURO_RESOURCE_INDEX => {
            crate::gacha::kuro::read_install_version(install_path, &edition.data_folder)
        }
        YOSTAR_FILE_INDEX => {
            crate::gacha::yostar::read_install_version(install_path, &edition.data_folder)
        }
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
        GRYPHLINE_RESOURCE_PATCH => {
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
        KURO_RESOURCE_INDEX | YOSTAR_FILE_INDEX => {
            let has_install = edition_exe_name(manifest, edition_id)
                .is_some_and(|exe| install_path.join(exe).exists());
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
    crate::gacha::art::resolve_art(manifest, "grid")
}

fn hoyo_edition_from_id(id: &str) -> Result<crate::gacha::hoyo::HoyoEdition> {
    use crate::gacha::hoyo::HoyoEdition;
    match id {
        "global" => Ok(HoyoEdition::Global),
        "china" => Ok(HoyoEdition::China),
        other => bail!("no hoyo edition for id: {}", other),
    }
}

fn hoyo_voices_from_ids(ids: &[String]) -> Vec<crate::gacha::hoyo::VoiceLocale> {
    use crate::gacha::hoyo::VoiceLocale;
    ids.iter()
        .filter_map(|id| {
            VoiceLocale::all()
                .iter()
                .find(|v| v.api_name() == id)
                .copied()
        })
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
        .ok_or_else(|| {
            anyhow!(
                "no biz_id in manifest {} for edition {}",
                manifest.id,
                edition_id
            )
        })
}
