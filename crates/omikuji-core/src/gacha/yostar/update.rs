use anyhow::Result;

use crate::gacha::manifest::GachaManifest;

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub game_slug: String,
    pub edition: String,
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
    pub can_diff: bool,
    pub delta_supported: bool,
}

// 0 like kuro's full sync: the real size depends on what already matches on disk
pub async fn check_for_update(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Result<Option<UpdateInfo>> {
    let Some(from_version) = super::installed_version(&manifest.game_slug, edition_id) else {
        return Ok(None);
    };
    let api = super::edition_api(manifest, edition_id)?;
    let config = super::api::fetch_game_config(&api).await?;
    if config.version == from_version {
        return Ok(None);
    }
    Ok(Some(UpdateInfo {
        game_slug: manifest.game_slug.clone(),
        edition: edition_id.to_string(),
        from_version,
        to_version: config.version,
        download_size: 0,
        can_diff: false,
        delta_supported: false,
    }))
}
