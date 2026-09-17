pub mod api;
pub mod sophon;
pub mod source;
pub mod update;

use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::gacha::manifest::GachaManifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoyoEdition {
    Global,
    China,
}

impl HoyoEdition {
    pub fn from_id(id: &str) -> Result<Self> {
        match id {
            "global" => Ok(Self::Global),
            "china" => Ok(Self::China),
            other => bail!("unknown hoyo edition: {}", other),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::China => "china",
        }
    }

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

    pub fn from_api_name(name: &str) -> Option<Self> {
        Self::all().iter().find(|v| v.api_name() == name).copied()
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

pub fn biz_id(manifest: &GachaManifest, edition_id: &str) -> Result<String> {
    manifest
        .edition(edition_id)
        .and_then(|e| e.strategy_config.get("biz_id"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow!(
                "no biz_id in manifest {} for edition {}",
                manifest.id,
                edition_id
            )
        })
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
