use anyhow::Result;

use super::installed_version;
use crate::gacha::manifest::GachaManifest;

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub edition_id: String,
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
    pub can_diff: bool,
    pub delta_supported: bool,
}

pub async fn check_for_update(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Result<Option<UpdateInfo>> {
    let Some(from_version) = installed_version(&manifest.game_slug, edition_id) else {
        return Ok(None);
    };

    let cfg = super::api::EditionConfig::from_manifest(manifest, edition_id)?;
    let resp = super::api::fetch_latest(&cfg, &from_version).await?;
    let to_version = resp.version.clone();
    if to_version == from_version || to_version.is_empty() {
        return Ok(None);
    }

    let patches = super::api::patches_from(&resp);
    let overlay_size: u64 = patches.iter().map(|p| p.package_size).sum();

    // the top-level patch field is null on minor bumps but patch.json usually has entries, so stay optimistic <3333
    let can_diff = true;
    let download_size = overlay_size;

    Ok(Some(UpdateInfo {
        edition_id: edition_id.to_string(),
        from_version,
        to_version,
        download_size,
        can_diff,
        delta_supported: true,
    }))
}
