use anyhow::Result;

use super::sophon;
use super::{HoyoEdition, installed_version};
use crate::gacha::manifest::{GachaManifest, load_all};
use crate::gacha::strategies::HOYO_SOPHON;

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

pub async fn check_all_installed() -> Vec<UpdateInfo> {
    let mut out = Vec::new();
    for manifest in hoyo_manifests() {
        for edition in manifest.editions {
            let Some(edition_enum) = parse_edition(&edition.id) else {
                continue;
            };
            if installed_version(&manifest.game_slug, edition_enum).is_none() {
                continue;
            }
            let Some(biz_id) = edition
                .strategy_config
                .get("biz_id")
                .and_then(|v| v.as_str())
            else {
                continue;
            };
            if let Ok(Some(info)) =
                check_for_update(biz_id, &manifest.game_slug, edition_enum).await
            {
                out.push(info);
            }
        }
    }
    out
}

pub async fn check_by_app_id(app_id: &str) -> Option<UpdateInfo> {
    let (manifest, edition_id, _) = crate::gacha::strategies::find_for_app_id(app_id)?;
    let edition = parse_edition(&edition_id)?;
    let biz_id = manifest
        .editions
        .iter()
        .find(|e| e.id == edition_id)?
        .strategy_config
        .get("biz_id")?
        .as_str()?;
    match check_for_update(biz_id, &manifest.game_slug, edition).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("update check for {} failed: {}", app_id, e);
            None
        }
    }
}

pub fn update_app_id(info: &UpdateInfo) -> String {
    format!(
        "{}:{}",
        info.game_slug,
        match info.edition {
            HoyoEdition::Global => "global",
            HoyoEdition::China => "china",
        }
    )
}

pub fn current_version(app_id: &str) -> Option<String> {
    let (manifest, edition_id, _) = crate::gacha::strategies::find_for_app_id(app_id)?;
    let edition = parse_edition(&edition_id)?;
    installed_version(&manifest.game_slug, edition)
}

fn hoyo_manifests() -> Vec<GachaManifest> {
    load_all()
        .into_iter()
        .filter(|m| m.install_strategy == HOYO_SOPHON)
        .collect()
}

fn parse_edition(id: &str) -> Option<HoyoEdition> {
    match id {
        "global" => Some(HoyoEdition::Global),
        "china" => Some(HoyoEdition::China),
        _ => None,
    }
}
