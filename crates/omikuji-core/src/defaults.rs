// no runtime cascade; only seeded into a Game at creation or via apply-to-existing

use crate::library::{Game, GraphicsConfig, LaunchConfig, SystemConfig, WineConfig};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
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
    pub prefix: Option<String>,
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
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub dll_overrides: IndexMap<String, String>,
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
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub env: IndexMap<String, String>,
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
    pub refresh_rate: Option<u32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discord_rpc: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigSections {
    pub wine: WineConfig,
    pub launch: LaunchConfig,
    pub graphics: GraphicsConfig,
    pub system: SystemConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMode {
    Seed,
    Merge,
    Replace,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct DefaultField {
    pub key: &'static str,
    pub group: &'static str,
    pub is_map: bool,
}

pub trait DefaultSlot<T> {
    fn copy_into(&self, field: &mut T, stock: &T, mode: CopyMode);
}

impl<T: PartialEq + Clone> DefaultSlot<T> for Option<T> {
    fn copy_into(&self, field: &mut T, stock: &T, mode: CopyMode) {
        match (self, mode) {
            (None, CopyMode::Seed) => {}
            (Some(v), CopyMode::Seed) => {
                if field == stock {
                    *field = v.clone();
                }
            }
            (v, _) => *field = v.as_ref().unwrap_or(stock).clone(),
        }
    }
}

impl DefaultSlot<IndexMap<String, String>> for IndexMap<String, String> {
    fn copy_into(
        &self,
        field: &mut IndexMap<String, String>,
        _stock: &IndexMap<String, String>,
        mode: CopyMode,
    ) {
        match mode {
            CopyMode::Seed => {
                for (k, v) in self {
                    field.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
            CopyMode::Merge => field.extend(self.iter().map(|(k, v)| (k.clone(), v.clone()))),
            CopyMode::Replace => *field = self.clone(),
        }
    }
}

#[macro_export]
macro_rules! with_default_fields {
    ($then:ident) => {
        $then! {
            "wine.version" => str, wine, wine.version,
            "wine.prefix" => str, wine, wine.prefix,
            "wine.prefix_arch" => str, wine, wine.prefix_arch,
            "wine.esync" => bool, sync, wine.esync,
            "wine.fsync" => bool, sync, wine.fsync,
            "wine.ntsync" => bool, sync, wine.ntsync,
            "wine.dxvk" => bool, translation_layers, wine.dxvk,
            "wine.dxvk_version" => str, translation_layers, wine.dxvk_version,
            "wine.vkd3d" => bool, translation_layers, wine.vkd3d,
            "wine.vkd3d_version" => str, translation_layers, wine.vkd3d_version,
            "wine.dxvk_nvapi" => bool, translation_layers, wine.dxvk_nvapi,
            "wine.dxvk_nvapi_version" => str, translation_layers, wine.dxvk_nvapi_version,
            "wine.battleye" => bool, compatibility, wine.battleye,
            "wine.easyanticheat" => bool, compatibility, wine.easyanticheat,
            "wine.fsr" => bool, compatibility, wine.fsr,
            "wine.dpi_scaling" => bool, display, wine.dpi_scaling,
            "wine.dpi" => int, display, wine.dpi,
            "wine.audio_driver" => str, drivers, wine.audio_driver,
            "wine.graphics_driver" => str, drivers, wine.graphics_driver,
            "wine.dll_overrides" => map, dll_overrides, wine.dll_overrides,

            "launch.command_prefix" => str, launch, launch.command_prefix,
            "launch.env" => map, environment, launch.env,

            "graphics.mangohud" => bool, graphics, graphics.mangohud,
            "graphics.gpu" => str, graphics, graphics.gpu,

            "graphics.gamescope.enabled" => bool, gamescope, graphics.gamescope.enabled,
            "graphics.gamescope.width" => int, gamescope, graphics.gamescope.width,
            "graphics.gamescope.height" => int, gamescope, graphics.gamescope.height,
            "graphics.gamescope.game_width" => int, gamescope, graphics.gamescope.game_width,
            "graphics.gamescope.game_height" => int, gamescope, graphics.gamescope.game_height,
            "graphics.gamescope.fullscreen" => bool, gamescope, graphics.gamescope.fullscreen,
            "graphics.gamescope.borderless" => bool, gamescope, graphics.gamescope.borderless,
            "graphics.gamescope.integer_scaling" => bool, gamescope, graphics.gamescope.integer_scaling,
            "graphics.gamescope.hdr" => bool, gamescope, graphics.gamescope.hdr,
            "graphics.gamescope.fps" => int, gamescope, graphics.gamescope.fps,
            "graphics.gamescope.refresh_rate" => int, gamescope, graphics.gamescope.refresh_rate,
            "graphics.gamescope.filter" => str, gamescope, graphics.gamescope.filter,
            "graphics.gamescope.fsr_sharpness" => int, gamescope, graphics.gamescope.fsr_sharpness,

            "system.gamemode" => bool, performance, system.gamemode,
            "system.cpu_limit" => int, performance, system.cpu_limit,
            "system.pulse_latency" => bool, audio, system.pulse_latency,
            "system.prevent_sleep" => bool, power, system.prevent_sleep,
            "system.discord_rpc" => bool, discord, system.discord_rpc,
        }
    };
}

macro_rules! is_map_kind {
    (map) => {
        true
    };
    ($other:ident) => {
        false
    };
}

macro_rules! impl_default_fields {
    ($( $key:literal => $kind:ident, $group:ident, $($path:ident).+ ),* $(,)?) => {
        pub const FIELDS: &[DefaultField] = &[
            $( DefaultField { key: $key, group: stringify!($group), is_map: is_map_kind!($kind) } ),*
        ];

        impl Defaults {
            fn copy_field(&self, key: &str, game: &mut Game, stock: &ConfigSections, mode: CopyMode) {
                match key {
                    $( $key => self.$($path).+.copy_into(&mut game.$($path).+, &stock.$($path).+, mode), )*
                    _ => tracing::warn!("unknown defaults key: {key}"),
                }
            }
        }
    };
}

with_default_fields!(impl_default_fields);

pub fn defaults_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omikuji")
        .join("defaults.toml")
}

impl Defaults {
    pub fn apply_to<'a>(
        &self,
        game: &mut Game,
        keys: impl IntoIterator<Item = &'a str>,
        mode: CopyMode,
    ) {
        let stock = ConfigSections::default();
        for key in keys {
            self.copy_field(key, game, &stock, mode);
        }
    }

    pub fn load() -> Self {
        let path = defaults_path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(body) => toml::from_str::<Defaults>(&body).unwrap_or_else(|e| {
                tracing::warn!(
                    "couldn't parse {}: {} - using empty defaults",
                    path.display(),
                    e
                );
                Self::default()
            }),
            Err(e) => {
                tracing::warn!(
                    "couldn't read {}: {} - using empty defaults",
                    path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let body = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        crate::fs_util::write_atomic(&defaults_path(), body)
    }
}
