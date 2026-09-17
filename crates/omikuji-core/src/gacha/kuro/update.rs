use anyhow::Result;

use crate::gacha::manifest::GachaManifest;
use crate::gacha::state;
use crate::gacha::strategies::UpdateCheck;

pub async fn check_for_update(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Result<Option<UpdateCheck>> {
    let Some(from_version) = state::read_installed_version(&manifest.game_slug, edition_id) else {
        return Ok(None);
    };
    let info = super::api::fetch_resource_info(manifest, edition_id).await?;
    if info.version == from_version || info.version.is_empty() {
        return Ok(None);
    }
    let matched = info.matching_patch(&from_version);
    let download_size = matched.map(|p| p.download_size).unwrap_or(0);
    let has_delta = matched.is_some();
    Ok(Some(UpdateCheck {
        from_version,
        to_version: info.version,
        download_size,
        can_diff: has_delta,
        delta_supported: has_delta,
    }))
}
