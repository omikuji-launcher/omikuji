use anyhow::Result;

use super::sophon;
use super::{HoyoEdition, installed_version};

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub game_slug: String,
    pub edition: HoyoEdition,
    pub from_version: String,
    pub to_version: String,
    pub download_size: u64,
    pub can_diff: bool,
    pub delta_supported: bool,
}

pub async fn check_for_update(
    biz_id: &str,
    game_slug: &str,
    edition: HoyoEdition,
) -> Result<Option<UpdateInfo>> {
    let Some(from_version) = installed_version(game_slug, edition) else {
        return Ok(None);
    };

    let branches = sophon::api::fetch_game_branches(edition).await?;
    let Some(branch) = branches.find_for(biz_id) else {
        return Ok(None);
    };
    let Some(main) = &branch.main else {
        return Ok(None);
    };

    let target = crate::gacha::strategies::normalize_version(&from_version);
    if crate::gacha::strategies::normalize_version(&main.tag) == target {
        return Ok(None);
    }

    let matched_tag = main
        .diff_tags
        .iter()
        .find(|t| crate::gacha::strategies::normalize_version(t) == target)
        .cloned();
    let can_diff = matched_tag.is_some();

    let download_size = if let Some(tag) = matched_tag {
        match sophon::api::fetch_patch_build(edition, main).await {
            Ok(diffs) => diffs
                .get_for("game")
                .and_then(|d| d.stats.get(&tag))
                .and_then(|s| s.compressed_size.parse::<u64>().ok())
                .unwrap_or(0),
            Err(_) => 0,
        }
    } else {
        0
    };

    let delta_supported = !main.diff_tags.is_empty();

    Ok(Some(UpdateInfo {
        game_slug: game_slug.to_string(),
        edition,
        from_version,
        to_version: main.tag.clone(),
        download_size,
        can_diff,
        delta_supported,
    }))
}
