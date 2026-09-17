use anyhow::Result;

use super::HoyoEdition;
use super::sophon;
use crate::gacha::state;
use crate::gacha::strategies::UpdateCheck;

pub async fn check_for_update(
    biz_id: &str,
    game_slug: &str,
    edition: HoyoEdition,
) -> Result<Option<UpdateCheck>> {
    let Some(from_version) = state::read_installed_version(game_slug, edition.id()) else {
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

    Ok(Some(UpdateCheck {
        from_version,
        to_version: main.tag.clone(),
        download_size,
        can_diff,
        delta_supported,
    }))
}
