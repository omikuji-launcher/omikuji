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

    let cfg = super::api::EditionConfig::from_manifest(manifest, edition_id)?;
    let resp = super::api::fetch_latest(&cfg, &from_version).await?;
    let to_version = resp.version.clone();
    if to_version == from_version || to_version.is_empty() {
        return Ok(None);
    }

    let download_size: u64 = super::api::patches_from(&resp)
        .iter()
        .map(|p| p.package_size)
        .sum();

    Ok(Some(UpdateCheck {
        from_version,
        to_version,
        download_size,
        can_diff: true,
        delta_supported: true,
    }))
}
