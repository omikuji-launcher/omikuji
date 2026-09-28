use anyhow::Result;

use super::HoyoEdition;
use super::sophon;
use crate::gacha::state;
use crate::gacha::strategies::{UpdateCheck, normalize_version};
use crate::process::UpdateKind;

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

    let installed = normalize_version(&from_version);
    let (package, kind) = if normalize_version(&main.tag) != installed {
        (main, UpdateKind::Required)
    } else if let Some(pre_download) = &branch.pre_download {
        (pre_download, UpdateKind::PreDownload)
    } else {
        return Ok(None);
    };

    let matched_tag = package
        .diff_tags
        .iter()
        .find(|t| normalize_version(t) == installed)
        .cloned();
    if kind == UpdateKind::PreDownload && matched_tag.is_none() {
        return Ok(None);
    }
    let can_diff = matched_tag.is_some();

    let download_size = if let Some(tag) = matched_tag {
        match sophon::api::fetch_patch_build(edition, package).await {
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

    let delta_supported = !package.diff_tags.is_empty();

    Ok(Some(UpdateCheck {
        from_version,
        to_version: package.tag.clone(),
        download_size,
        can_diff,
        delta_supported,
        kind,
    }))
}
