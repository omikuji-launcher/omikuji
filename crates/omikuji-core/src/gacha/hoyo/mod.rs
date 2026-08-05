pub mod api;
pub mod sophon;
pub mod source;
pub mod update;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// HoyoGame enum lived here to carry per-game biz_ids/data_folders/needs_patch as typed methods.
// dropped 2026-04-26: per-game data now lives in manifest.strategy_config (asset repo); strategy code reads from there.
// adding a new hoyo gacha = push manifest + push art + 1 line in gacha/index.json. zero rust touch.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoyoEdition {
    Global,
    China,
}

impl HoyoEdition {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::China => "China",
        }
    }

    pub fn api_base(&self) -> &'static str {
        match self {
            Self::Global => "https://sg-hyp-api.hoyoverse.com/hyp/hyp-connect/api",
            Self::China => "https://hyp-api.mihoyo.com/hyp/hyp-connect/api",
        }
    }

    pub fn launcher_id(&self) -> &'static str {
        match self {
            Self::Global => "VYTpXlbWo8",
            Self::China => "jGHBHlcOq1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoiceLocale {
    English,
    Japanese,
    Korean,
    Chinese,
}

impl VoiceLocale {
    pub fn all() -> &'static [VoiceLocale] {
        &[Self::English, Self::Japanese, Self::Korean, Self::Chinese]
    }

    pub fn api_name(&self) -> &'static str {
        match self {
            Self::English => "en-us",
            Self::Japanese => "ja-jp",
            Self::Korean => "ko-kr",
            Self::Chinese => "zh-cn",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Japanese => "Japanese",
            Self::Korean => "Korean",
            Self::Chinese => "Chinese",
        }
    }

    pub fn folder_name(&self) -> &'static str {
        match self {
            Self::English => "English(US)",
            Self::Japanese => "Japanese",
            Self::Korean => "Korean",
            Self::Chinese => "Chinese",
        }
    }
}

pub fn parse_voice_csv(csv: &str) -> Vec<VoiceLocale> {
    csv.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            VoiceLocale::all()
                .iter()
                .find(|v| v.api_name() == s)
                .copied()
        })
        .collect()
}

const PUBLISHER_SLUG: &str = "hoyoverse";

fn edition_id(edition: HoyoEdition) -> &'static str {
    match edition {
        HoyoEdition::Global => "global",
        HoyoEdition::China => "china",
    }
}

pub fn version_file(game_slug: &str, edition: HoyoEdition) -> PathBuf {
    crate::gacha::state::version_file(PUBLISHER_SLUG, game_slug, edition_id(edition))
}

pub fn installed_version(game_slug: &str, edition: HoyoEdition) -> Option<String> {
    crate::gacha::state::read_installed_version(PUBLISHER_SLUG, game_slug, edition_id(edition))
}

pub fn set_installed_version(game_slug: &str, edition: HoyoEdition, version: &str) {
    crate::gacha::state::write_installed_version(
        PUBLISHER_SLUG,
        game_slug,
        edition_id(edition),
        version,
    );
}

pub fn read_install_version(install_path: &std::path::Path, data_folder: &str) -> Option<String> {
    use crate::gacha::state;
    if let Some(v) = state::read_install_dotversion(install_path) {
        return Some(v);
    }
    if let Some(v) = state::scan_globalgamemanagers(install_path, data_folder, b'_') {
        return Some(v);
    }
    if let Some(v) = state::scan_globalgamemanagers(install_path, data_folder, 0) {
        return Some(v);
    }
    if data_folder.is_empty() {
        return None;
    }
    state::scan_unity_file(
        &install_path.join(data_folder).join("data.unity3d"),
        2000,
        524288,
        0,
    )
}
