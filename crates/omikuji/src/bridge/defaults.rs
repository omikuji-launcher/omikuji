use cxx_qt::{CxxQtType, Threading};
use omikuji_core::defaults::{Defaults, defaults_path};
use omikuji_core::fs_watcher::FileWatcher;
use omikuji_core::library::{
    GamescopeConfig, GraphicsConfig, LaunchConfig, SystemConfig, WineConfig,
};
use std::pin::Pin;
use std::time::{Duration, Instant};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type DefaultsBridge = super::DefaultsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "changed"]
        fn changed(self: Pin<&mut DefaultsBridge>);
    }

    impl cxx_qt::Threading for DefaultsBridge {}

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_name = "getConfig"]
        fn get_config(self: &DefaultsBridge) -> QMap_QString_QVariant;

        #[qinvokable]
        #[cxx_name = "setKeysJson"]
        fn set_keys_json(self: &DefaultsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "populatedSectionsJson"]
        fn populated_sections_json(self: &DefaultsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "updateField"]
        fn update_field(self: Pin<&mut DefaultsBridge>, key: &QString, value: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "resetField"]
        fn reset_field(self: Pin<&mut DefaultsBridge>, key: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "initWatcher"]
        fn init_watcher(self: Pin<&mut DefaultsBridge>);
    }
}

pub struct DefaultsRust {
    pub data: Defaults,
    pub watcher: Option<FileWatcher>,
    pub suppress_reload_until: Option<Instant>,
}

impl Default for DefaultsRust {
    fn default() -> Self {
        Self {
            data: Defaults::load(),
            watcher: None,
            suppress_reload_until: None,
        }
    }
}

impl qobject::DefaultsBridge {
    fn get_config(&self) -> cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant> {
        use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QVariant};
        use omikuji_core::library::WineConfig;

        let mut m = QMap::<QMapPair_QString_QVariant>::default();
        let d = &self.data;
        let w = WineConfig::default();

        macro_rules! put_str {
            ($k:expr, $v:expr) => {
                m.insert(QString::from($k), QVariant::from(&QString::from(&*$v)));
            };
        }
        macro_rules! put_bool {
            ($k:expr, $v:expr) => {
                m.insert(QString::from($k), QVariant::from(&$v));
            };
        }
        macro_rules! put_int {
            ($k:expr, $v:expr) => {
                m.insert(QString::from($k), QVariant::from(&($v as i32)));
            };
        }

        put_str!("wine.version", d.wine.version.clone().unwrap_or(w.version));
        put_str!("wine.prefix_arch", d.wine.prefix_arch.clone().unwrap_or(w.prefix_arch));
        put_bool!("wine.esync", d.wine.esync.unwrap_or(w.esync));
        put_bool!("wine.fsync", d.wine.fsync.unwrap_or(w.fsync));
        put_bool!("wine.ntsync", d.wine.ntsync.unwrap_or(w.ntsync));
        put_bool!("wine.dxvk", d.wine.dxvk.unwrap_or(w.dxvk));
        put_str!("wine.dxvk_version", d.wine.dxvk_version.clone().unwrap_or(w.dxvk_version));
        put_bool!("wine.vkd3d", d.wine.vkd3d.unwrap_or(w.vkd3d));
        put_str!("wine.vkd3d_version", d.wine.vkd3d_version.clone().unwrap_or(w.vkd3d_version));
        put_bool!("wine.d3d_extras", d.wine.d3d_extras.unwrap_or(w.d3d_extras));
        put_str!("wine.d3d_extras_version", d.wine.d3d_extras_version.clone().unwrap_or(w.d3d_extras_version));
        put_bool!("wine.dxvk_nvapi", d.wine.dxvk_nvapi.unwrap_or(w.dxvk_nvapi));
        put_str!("wine.dxvk_nvapi_version", d.wine.dxvk_nvapi_version.clone().unwrap_or(w.dxvk_nvapi_version));
        put_bool!("wine.fsr", d.wine.fsr.unwrap_or(w.fsr));
        put_bool!("wine.battleye", d.wine.battleye.unwrap_or(w.battleye));
        put_bool!("wine.easyanticheat", d.wine.easyanticheat.unwrap_or(w.easyanticheat));
        put_bool!("wine.dpi_scaling", d.wine.dpi_scaling.unwrap_or(w.dpi_scaling));
        put_int!("wine.dpi", d.wine.dpi.unwrap_or(w.dpi));
        put_str!("wine.audio_driver", d.wine.audio_driver.clone().unwrap_or(w.audio_driver));
        put_str!("wine.graphics_driver", d.wine.graphics_driver.clone().unwrap_or(w.graphics_driver));
        if let Ok(json) = serde_json::to_string(&d.wine.dll_overrides) {
            put_str!("wine.dll_overrides", json);
        }

        put_str!("launch.command_prefix", d.launch.command_prefix.clone().unwrap_or_default());
        if let Ok(json) = serde_json::to_string(&d.launch.env) {
            put_str!("launch.env", json);
        }

        put_bool!("graphics.mangohud", d.graphics.mangohud.unwrap_or(false));
        put_str!("graphics.gpu", d.graphics.gpu.clone().unwrap_or_default());

        let gs = &d.graphics.gamescope;
        put_bool!("graphics.gamescope.enabled", gs.enabled.unwrap_or(false));
        put_int!("graphics.gamescope.width", gs.width.unwrap_or(0));
        put_int!("graphics.gamescope.height", gs.height.unwrap_or(0));
        put_int!("graphics.gamescope.game_width", gs.game_width.unwrap_or(0));
        put_int!("graphics.gamescope.game_height", gs.game_height.unwrap_or(0));
        put_int!("graphics.gamescope.fps", gs.fps.unwrap_or(0));
        put_bool!("graphics.gamescope.fullscreen", gs.fullscreen.unwrap_or(false));
        put_bool!("graphics.gamescope.borderless", gs.borderless.unwrap_or(false));
        put_bool!("graphics.gamescope.integer_scaling", gs.integer_scaling.unwrap_or(false));
        put_bool!("graphics.gamescope.hdr", gs.hdr.unwrap_or(false));
        put_str!("graphics.gamescope.filter", gs.filter.clone().unwrap_or_default());
        put_int!("graphics.gamescope.fsr_sharpness", gs.fsr_sharpness.unwrap_or(0));

        put_bool!("system.gamemode", d.system.gamemode.unwrap_or(false));
        put_bool!("system.prevent_sleep", d.system.prevent_sleep.unwrap_or(false));
        put_bool!("system.pulse_latency", d.system.pulse_latency.unwrap_or(false));
        put_int!("system.cpu_limit", d.system.cpu_limit.unwrap_or(0));

        m
    }

    fn set_keys_json(&self) -> cxx_qt_lib::QString {
        let keys = collect_set_keys(&self.data);
        let json = serde_json::to_string(&keys).unwrap_or_else(|_| "[]".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn populated_sections_json(&self) -> cxx_qt_lib::QString {
        let sections = self.data.populated_sections();
        let json = serde_json::to_string(&sections).unwrap_or_else(|_| "[]".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn update_field(mut self: Pin<&mut Self>, key: &cxx_qt_lib::QString, value: &cxx_qt_lib::QString) -> bool {
        let k = key.to_string();
        let v = value.to_string();
        let d = &mut self.as_mut().rust_mut().get_mut().data;
        let ok = apply_to_defaults(d, &k, &v);
        if ok {
            self.as_mut().persist();
            self.as_mut().changed();
        }
        ok
    }

    fn reset_field(mut self: Pin<&mut Self>, key: &cxx_qt_lib::QString) -> bool {
        let k = key.to_string();
        let d = &mut self.as_mut().rust_mut().get_mut().data;
        let ok = clear_in_defaults(d, &k);
        if ok {
            self.as_mut().persist();
            self.as_mut().changed();
        }
        ok
    }

    fn persist(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().suppress_reload_until =
            Some(Instant::now() + Duration::from_millis(600));
        if let Err(e) = self.as_ref().rust().data.save() {
            eprintln!("[defaults] save failed: {}", e);
        }
    }

    fn init_watcher(mut self: Pin<&mut Self>) {
        if self.as_ref().rust().watcher.is_some() {
            return;
        }
        let path = defaults_path();
        let qt_thread = self.as_mut().qt_thread();
        let watcher = FileWatcher::watch(path, move || {
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::DefaultsBridge>| {
                let within_window = obj
                    .as_ref()
                    .rust()
                    .suppress_reload_until
                    .map(|until| Instant::now() < until)
                    .unwrap_or(false);
                if within_window {
                    return;
                }
                obj.as_mut().rust_mut().get_mut().data = Defaults::load();
                obj.as_mut().changed();
            });
        });
        match watcher {
            Ok(w) => {
                self.as_mut().rust_mut().get_mut().watcher = Some(w);
                eprintln!("[defaults] watching {} via notify", defaults_path().display());
            }
            Err(e) => eprintln!("[defaults] failed to start watcher: {e}"),
        }
    }
}

fn parse_bool(s: &str) -> bool { s == "true" }
fn parse_u32(s: &str) -> u32 { s.parse().unwrap_or(0) }

fn apply_to_defaults(d: &mut Defaults, key: &str, value: &str) -> bool {
    match key {
        "wine.version" => d.wine.version = Some(value.to_string()),
        "wine.prefix_arch" => d.wine.prefix_arch = Some(value.to_string()),
        "wine.esync" => d.wine.esync = Some(parse_bool(value)),
        "wine.fsync" => d.wine.fsync = Some(parse_bool(value)),
        "wine.ntsync" => d.wine.ntsync = Some(parse_bool(value)),
        "wine.dxvk" => d.wine.dxvk = Some(parse_bool(value)),
        "wine.dxvk_version" => d.wine.dxvk_version = Some(value.to_string()),
        "wine.vkd3d" => d.wine.vkd3d = Some(parse_bool(value)),
        "wine.vkd3d_version" => d.wine.vkd3d_version = Some(value.to_string()),
        "wine.d3d_extras" => d.wine.d3d_extras = Some(parse_bool(value)),
        "wine.d3d_extras_version" => d.wine.d3d_extras_version = Some(value.to_string()),
        "wine.dxvk_nvapi" => d.wine.dxvk_nvapi = Some(parse_bool(value)),
        "wine.dxvk_nvapi_version" => d.wine.dxvk_nvapi_version = Some(value.to_string()),
        "wine.fsr" => d.wine.fsr = Some(parse_bool(value)),
        "wine.battleye" => d.wine.battleye = Some(parse_bool(value)),
        "wine.easyanticheat" => d.wine.easyanticheat = Some(parse_bool(value)),
        "wine.dpi_scaling" => d.wine.dpi_scaling = Some(parse_bool(value)),
        "wine.dpi" => d.wine.dpi = Some(parse_u32(value)),
        "wine.audio_driver" => d.wine.audio_driver = Some(value.to_string()),
        "wine.graphics_driver" => d.wine.graphics_driver = Some(value.to_string()),
        "wine.dll_overrides" => {
            if let Ok(map) = serde_json::from_str(value) {
                d.wine.dll_overrides = map;
            } else { return false; }
        }

        "launch.command_prefix" => d.launch.command_prefix = Some(value.to_string()),
        "launch.env" => {
            if let Ok(env) = serde_json::from_str(value) {
                d.launch.env = env;
            } else { return false; }
        }

        "graphics.mangohud" => d.graphics.mangohud = Some(parse_bool(value)),
        "graphics.gpu" => d.graphics.gpu = Some(value.to_string()),

        "graphics.gamescope.enabled" => d.graphics.gamescope.enabled = Some(parse_bool(value)),
        "graphics.gamescope.width" => d.graphics.gamescope.width = Some(parse_u32(value)),
        "graphics.gamescope.height" => d.graphics.gamescope.height = Some(parse_u32(value)),
        "graphics.gamescope.game_width" => d.graphics.gamescope.game_width = Some(parse_u32(value)),
        "graphics.gamescope.game_height" => d.graphics.gamescope.game_height = Some(parse_u32(value)),
        "graphics.gamescope.fps" => d.graphics.gamescope.fps = Some(parse_u32(value)),
        "graphics.gamescope.fullscreen" => d.graphics.gamescope.fullscreen = Some(parse_bool(value)),
        "graphics.gamescope.borderless" => d.graphics.gamescope.borderless = Some(parse_bool(value)),
        "graphics.gamescope.integer_scaling" => d.graphics.gamescope.integer_scaling = Some(parse_bool(value)),
        "graphics.gamescope.hdr" => d.graphics.gamescope.hdr = Some(parse_bool(value)),
        "graphics.gamescope.filter" => d.graphics.gamescope.filter = Some(value.to_string()),
        "graphics.gamescope.fsr_sharpness" => d.graphics.gamescope.fsr_sharpness = Some(parse_u32(value)),

        "system.gamemode" => d.system.gamemode = Some(parse_bool(value)),
        "system.prevent_sleep" => d.system.prevent_sleep = Some(parse_bool(value)),
        "system.pulse_latency" => d.system.pulse_latency = Some(parse_bool(value)),
        "system.cpu_limit" => d.system.cpu_limit = Some(parse_u32(value)),

        _ => {
            eprintln!("[defaults] unknown key: {}", key);
            return false;
        }
    }
    true
}

fn clear_in_defaults(d: &mut Defaults, key: &str) -> bool {
    match key {
        "wine.version" => d.wine.version = None,
        "wine.prefix_arch" => d.wine.prefix_arch = None,
        "wine.esync" => d.wine.esync = None,
        "wine.fsync" => d.wine.fsync = None,
        "wine.ntsync" => d.wine.ntsync = None,
        "wine.dxvk" => d.wine.dxvk = None,
        "wine.dxvk_version" => d.wine.dxvk_version = None,
        "wine.vkd3d" => d.wine.vkd3d = None,
        "wine.vkd3d_version" => d.wine.vkd3d_version = None,
        "wine.d3d_extras" => d.wine.d3d_extras = None,
        "wine.d3d_extras_version" => d.wine.d3d_extras_version = None,
        "wine.dxvk_nvapi" => d.wine.dxvk_nvapi = None,
        "wine.dxvk_nvapi_version" => d.wine.dxvk_nvapi_version = None,
        "wine.fsr" => d.wine.fsr = None,
        "wine.battleye" => d.wine.battleye = None,
        "wine.easyanticheat" => d.wine.easyanticheat = None,
        "wine.dpi_scaling" => d.wine.dpi_scaling = None,
        "wine.dpi" => d.wine.dpi = None,
        "wine.audio_driver" => d.wine.audio_driver = None,
        "wine.graphics_driver" => d.wine.graphics_driver = None,
        "wine.dll_overrides" => d.wine.dll_overrides.clear(),
        "launch.command_prefix" => d.launch.command_prefix = None,
        "launch.env" => d.launch.env.clear(),
        "graphics.mangohud" => d.graphics.mangohud = None,
        "graphics.gpu" => d.graphics.gpu = None,
        "graphics.gamescope.enabled" => d.graphics.gamescope.enabled = None,
        "graphics.gamescope.width" => d.graphics.gamescope.width = None,
        "graphics.gamescope.height" => d.graphics.gamescope.height = None,
        "graphics.gamescope.game_width" => d.graphics.gamescope.game_width = None,
        "graphics.gamescope.game_height" => d.graphics.gamescope.game_height = None,
        "graphics.gamescope.fps" => d.graphics.gamescope.fps = None,
        "graphics.gamescope.fullscreen" => d.graphics.gamescope.fullscreen = None,
        "graphics.gamescope.borderless" => d.graphics.gamescope.borderless = None,
        "graphics.gamescope.integer_scaling" => d.graphics.gamescope.integer_scaling = None,
        "graphics.gamescope.hdr" => d.graphics.gamescope.hdr = None,
        "graphics.gamescope.filter" => d.graphics.gamescope.filter = None,
        "graphics.gamescope.fsr_sharpness" => d.graphics.gamescope.fsr_sharpness = None,
        "system.gamemode" => d.system.gamemode = None,
        "system.prevent_sleep" => d.system.prevent_sleep = None,
        "system.pulse_latency" => d.system.pulse_latency = None,
        "system.cpu_limit" => d.system.cpu_limit = None,
        _ => {
            eprintln!("[defaults] unknown key to reset: {}", key);
            return false;
        }
    }
    true
}

// equality with the hardcoded default = nothing to undo, no reset badge
fn collect_set_keys(d: &Defaults) -> Vec<String> {
    let mut k = Vec::new();
    let w = WineConfig::default();
    let g = GraphicsConfig::default();
    let gs_def = GamescopeConfig::default();
    let s = SystemConfig::default();
    let l = LaunchConfig::default();

    if d.wine.version.as_ref().is_some_and(|v| v != &w.version) { k.push("wine.version".into()); }
    if d.wine.prefix_arch.as_ref().is_some_and(|v| v != &w.prefix_arch) { k.push("wine.prefix_arch".into()); }
    if d.wine.esync.is_some_and(|v| v != w.esync) { k.push("wine.esync".into()); }
    if d.wine.fsync.is_some_and(|v| v != w.fsync) { k.push("wine.fsync".into()); }
    if d.wine.ntsync.is_some_and(|v| v != w.ntsync) { k.push("wine.ntsync".into()); }
    if d.wine.dxvk.is_some_and(|v| v != w.dxvk) { k.push("wine.dxvk".into()); }
    if d.wine.dxvk_version.as_ref().is_some_and(|v| v != &w.dxvk_version) { k.push("wine.dxvk_version".into()); }
    if d.wine.vkd3d.is_some_and(|v| v != w.vkd3d) { k.push("wine.vkd3d".into()); }
    if d.wine.vkd3d_version.as_ref().is_some_and(|v| v != &w.vkd3d_version) { k.push("wine.vkd3d_version".into()); }
    if d.wine.d3d_extras.is_some_and(|v| v != w.d3d_extras) { k.push("wine.d3d_extras".into()); }
    if d.wine.d3d_extras_version.as_ref().is_some_and(|v| v != &w.d3d_extras_version) { k.push("wine.d3d_extras_version".into()); }
    if d.wine.dxvk_nvapi.is_some_and(|v| v != w.dxvk_nvapi) { k.push("wine.dxvk_nvapi".into()); }
    if d.wine.dxvk_nvapi_version.as_ref().is_some_and(|v| v != &w.dxvk_nvapi_version) { k.push("wine.dxvk_nvapi_version".into()); }
    if d.wine.fsr.is_some_and(|v| v != w.fsr) { k.push("wine.fsr".into()); }
    if d.wine.battleye.is_some_and(|v| v != w.battleye) { k.push("wine.battleye".into()); }
    if d.wine.easyanticheat.is_some_and(|v| v != w.easyanticheat) { k.push("wine.easyanticheat".into()); }
    if d.wine.dpi_scaling.is_some_and(|v| v != w.dpi_scaling) { k.push("wine.dpi_scaling".into()); }
    if d.wine.dpi.is_some_and(|v| v != w.dpi) { k.push("wine.dpi".into()); }
    if d.wine.audio_driver.as_ref().is_some_and(|v| v != &w.audio_driver) { k.push("wine.audio_driver".into()); }
    if d.wine.graphics_driver.as_ref().is_some_and(|v| v != &w.graphics_driver) { k.push("wine.graphics_driver".into()); }
    if !d.wine.dll_overrides.is_empty() { k.push("wine.dll_overrides".into()); }

    if d.launch.command_prefix.as_ref().is_some_and(|v| v != &l.command_prefix) { k.push("launch.command_prefix".into()); }
    if !d.launch.env.is_empty() { k.push("launch.env".into()); }

    if d.graphics.mangohud.is_some_and(|v| v != g.mangohud) { k.push("graphics.mangohud".into()); }
    if d.graphics.gpu.as_ref().is_some_and(|v| v != &g.gpu) { k.push("graphics.gpu".into()); }

    let dgs = &d.graphics.gamescope;
    if dgs.enabled.is_some_and(|v| v != gs_def.enabled) { k.push("graphics.gamescope.enabled".into()); }
    if dgs.width.is_some_and(|v| v != gs_def.width) { k.push("graphics.gamescope.width".into()); }
    if dgs.height.is_some_and(|v| v != gs_def.height) { k.push("graphics.gamescope.height".into()); }
    if dgs.game_width.is_some_and(|v| v != gs_def.game_width) { k.push("graphics.gamescope.game_width".into()); }
    if dgs.game_height.is_some_and(|v| v != gs_def.game_height) { k.push("graphics.gamescope.game_height".into()); }
    if dgs.fps.is_some_and(|v| v != gs_def.fps) { k.push("graphics.gamescope.fps".into()); }
    if dgs.fullscreen.is_some_and(|v| v != gs_def.fullscreen) { k.push("graphics.gamescope.fullscreen".into()); }
    if dgs.borderless.is_some_and(|v| v != gs_def.borderless) { k.push("graphics.gamescope.borderless".into()); }
    if dgs.integer_scaling.is_some_and(|v| v != gs_def.integer_scaling) { k.push("graphics.gamescope.integer_scaling".into()); }
    if dgs.hdr.is_some_and(|v| v != gs_def.hdr) { k.push("graphics.gamescope.hdr".into()); }
    if dgs.filter.as_ref().is_some_and(|v| v != &gs_def.filter) { k.push("graphics.gamescope.filter".into()); }
    if dgs.fsr_sharpness.is_some_and(|v| v != gs_def.fsr_sharpness) { k.push("graphics.gamescope.fsr_sharpness".into()); }

    if d.system.gamemode.is_some_and(|v| v != s.gamemode) { k.push("system.gamemode".into()); }
    if d.system.prevent_sleep.is_some_and(|v| v != s.prevent_sleep) { k.push("system.prevent_sleep".into()); }
    if d.system.pulse_latency.is_some_and(|v| v != s.pulse_latency) { k.push("system.pulse_latency".into()); }
    if d.system.cpu_limit.is_some_and(|v| v != s.cpu_limit) { k.push("system.cpu_limit".into()); }

    k
}
