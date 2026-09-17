use anyhow::Result;

use crate::gacha::manifest::GachaManifest;
use crate::gacha::state;
use crate::gacha::strategies::UpdateCheck;

// 0 like kuro's full sync: the real size depends on what already matches on disk
pub async fn check_for_update(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Result<Option<UpdateCheck>> {
    let Some(from_version) = state::read_installed_version(&manifest.game_slug, edition_id) else {
        return Ok(None);
    };
    let api = super::edition_api(manifest, edition_id)?;
    let config = super::api::fetch_game_config(&api).await?;
    if config.version == from_version {
        return Ok(None);
    }
    Ok(Some(UpdateCheck {
        from_version,
        to_version: config.version,
        download_size: 0,
        can_diff: false,
        delta_supported: false,
    }))
}
