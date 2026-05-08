// no runtime cascade; only seeded into a Game at creation or via apply-to-existing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Defaults {
    pub wine: WineDefaults,
    pub launch: LaunchDefaults,
    pub graphics: GraphicsDefaults,
    pub system: SystemDefaults,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct WineDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_arch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub esync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ntsync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dxvk: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dxvk_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vkd3d: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vkd3d_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d3d_extras: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d3d_extras_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dxvk_nvapi: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dxvk_nvapi_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battleye: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub easyanticheat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpi_scaling: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpi: Option<u32>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub dll_overrides: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphics_driver: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LaunchDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_prefix: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct GraphicsDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mangohud: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<String>,
    pub gamescope: GamescopeDefaults,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct GamescopeDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fullscreen: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub borderless: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integer_scaling: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hdr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsr_sharpness: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SystemDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gamemode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prevent_sleep: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulse_latency: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_limit: Option<u32>,
}

pub fn defaults_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omikuji")
        .join("defaults.toml")
}

impl Defaults {
    // replace_maps only meaningful for env/dll_overrides
    pub fn apply_sections_to(
        &self,
        game: &mut crate::library::Game,
        sections: &[String],
        replace_maps: bool,
    ) {
        let has = |s: &str| sections.iter().any(|x| x == s);

        if has("wine") {
            if let Some(v) = &self.wine.version { game.wine.version = v.clone(); }
            if let Some(v) = &self.wine.prefix_arch { game.wine.prefix_arch = v.clone(); }
        }
        if has("sync") {
            if let Some(v) = self.wine.esync { game.wine.esync = v; }
            if let Some(v) = self.wine.fsync { game.wine.fsync = v; }
            if let Some(v) = self.wine.ntsync { game.wine.ntsync = v; }
        }
        if has("translation_layers") {
            if let Some(v) = self.wine.dxvk { game.wine.dxvk = v; }
            if let Some(v) = &self.wine.dxvk_version { game.wine.dxvk_version = v.clone(); }
            if let Some(v) = self.wine.vkd3d { game.wine.vkd3d = v; }
            if let Some(v) = &self.wine.vkd3d_version { game.wine.vkd3d_version = v.clone(); }
            if let Some(v) = self.wine.d3d_extras { game.wine.d3d_extras = v; }
            if let Some(v) = &self.wine.d3d_extras_version { game.wine.d3d_extras_version = v.clone(); }
            if let Some(v) = self.wine.dxvk_nvapi { game.wine.dxvk_nvapi = v; }
            if let Some(v) = &self.wine.dxvk_nvapi_version { game.wine.dxvk_nvapi_version = v.clone(); }
        }
        if has("compatibility") {
            if let Some(v) = self.wine.battleye { game.wine.battleye = v; }
            if let Some(v) = self.wine.easyanticheat { game.wine.easyanticheat = v; }
            if let Some(v) = self.wine.fsr { game.wine.fsr = v; }
        }
        if has("display") {
            if let Some(v) = self.wine.dpi_scaling { game.wine.dpi_scaling = v; }
            if let Some(v) = self.wine.dpi { game.wine.dpi = v; }
        }
        if has("drivers") {
            if let Some(v) = &self.wine.audio_driver { game.wine.audio_driver = v.clone(); }
            if let Some(v) = &self.wine.graphics_driver { game.wine.graphics_driver = v.clone(); }
        }
        if has("dll_overrides") {
            if replace_maps {
                game.wine.dll_overrides = self.wine.dll_overrides.clone();
            } else {
                for (k, v) in &self.wine.dll_overrides {
                    game.wine.dll_overrides.insert(k.clone(), v.clone());
                }
            }
        }
        if has("launch")
            && let Some(v) = &self.launch.command_prefix { game.launch.command_prefix = v.clone(); }
        if has("environment") {
            if replace_maps {
                game.launch.env = self.launch.env.clone();
            } else {
                for (k, v) in &self.launch.env {
                    game.launch.env.insert(k.clone(), v.clone());
                }
            }
        }
        if has("graphics") {
            if let Some(v) = self.graphics.mangohud { game.graphics.mangohud = v; }
            if let Some(v) = &self.graphics.gpu { game.graphics.gpu = v.clone(); }
        }
        if has("gamescope") {
            let gs = &mut game.graphics.gamescope;
            let dgs = &self.graphics.gamescope;
            if let Some(v) = dgs.enabled { gs.enabled = v; }
            if let Some(v) = dgs.width { gs.width = v; }
            if let Some(v) = dgs.height { gs.height = v; }
            if let Some(v) = dgs.game_width { gs.game_width = v; }
            if let Some(v) = dgs.game_height { gs.game_height = v; }
            if let Some(v) = dgs.fps { gs.fps = v; }
            if let Some(v) = dgs.fullscreen { gs.fullscreen = v; }
            if let Some(v) = dgs.borderless { gs.borderless = v; }
            if let Some(v) = dgs.integer_scaling { gs.integer_scaling = v; }
            if let Some(v) = dgs.hdr { gs.hdr = v; }
            if let Some(v) = &dgs.filter { gs.filter = v.clone(); }
            if let Some(v) = dgs.fsr_sharpness { gs.fsr_sharpness = v; }
        }
        if has("performance") {
            if let Some(v) = self.system.gamemode { game.system.gamemode = v; }
            if let Some(v) = self.system.cpu_limit { game.system.cpu_limit = v; }
        }
        if has("audio")
            && let Some(v) = self.system.pulse_latency { game.system.pulse_latency = v; }
        if has("power")
            && let Some(v) = self.system.prevent_sleep { game.system.prevent_sleep = v; }
    }

    pub fn populated_sections(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.wine.version.is_some() || self.wine.prefix_arch.is_some() {
            out.push("wine".into());
        }
        if self.wine.esync.is_some() || self.wine.fsync.is_some() || self.wine.ntsync.is_some() {
            out.push("sync".into());
        }
        if self.wine.dxvk.is_some() || self.wine.dxvk_version.is_some()
            || self.wine.vkd3d.is_some() || self.wine.vkd3d_version.is_some()
            || self.wine.d3d_extras.is_some() || self.wine.d3d_extras_version.is_some()
            || self.wine.dxvk_nvapi.is_some() || self.wine.dxvk_nvapi_version.is_some()
        {
            out.push("translation_layers".into());
        }
        if self.wine.battleye.is_some() || self.wine.easyanticheat.is_some() || self.wine.fsr.is_some() {
            out.push("compatibility".into());
        }
        if self.wine.dpi_scaling.is_some() || self.wine.dpi.is_some() {
            out.push("display".into());
        }
        if self.wine.audio_driver.is_some() || self.wine.graphics_driver.is_some() {
            out.push("drivers".into());
        }
        if !self.wine.dll_overrides.is_empty() {
            out.push("dll_overrides".into());
        }
        if self.launch.command_prefix.is_some() {
            out.push("launch".into());
        }
        if !self.launch.env.is_empty() {
            out.push("environment".into());
        }
        if self.graphics.mangohud.is_some() || self.graphics.gpu.is_some() {
            out.push("graphics".into());
        }
        let gs = &self.graphics.gamescope;
        if gs.enabled.is_some() || gs.width.is_some() || gs.height.is_some()
            || gs.game_width.is_some() || gs.game_height.is_some() || gs.fps.is_some()
            || gs.fullscreen.is_some() || gs.borderless.is_some()
            || gs.integer_scaling.is_some() || gs.hdr.is_some()
            || gs.filter.is_some() || gs.fsr_sharpness.is_some()
        {
            out.push("gamescope".into());
        }
        if self.system.gamemode.is_some() || self.system.cpu_limit.is_some() {
            out.push("performance".into());
        }
        if self.system.pulse_latency.is_some() {
            out.push("audio".into());
        }
        if self.system.prevent_sleep.is_some() {
            out.push("power".into());
        }
        out
    }

    pub fn load() -> Self {
        let path = defaults_path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(body) => toml::from_str::<Defaults>(&body).unwrap_or_else(|e| {
                eprintln!(
                    "[defaults] couldn't parse {}: {} — using empty defaults",
                    path.display(),
                    e
                );
                Self::default()
            }),
            Err(e) => {
                eprintln!(
                    "[defaults] couldn't read {}: {} — using empty defaults",
                    path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = defaults_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = toml::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, body)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }
}
