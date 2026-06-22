// ui preferences. live-mutable, app-written (via the bridge).
//
// separate from settings.toml on purpose: settings.toml is hacker knobs (paths, remote urls)
// that the user edits and the app reads once at startup. ui.toml is the app's own scratch
// for zom/tabs/layout prefs; app writes, qml reads live through the bridge, no restart needed.


use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct UiSettings {
    pub library: LibrarySettings,
    pub tabs: TabsSettings,
    pub nav: NavSettings,
    pub behavior: BehaviorSettings,
    pub display: DisplaySettings,
    pub theme: ThemeSettings,
    #[serde(default)]
    pub console_mode: ConsoleModeSettings,
    #[serde(default = "default_categories")]
    pub categories: Vec<CategoryEntry>,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            library: LibrarySettings::default(),
            tabs: TabsSettings::default(),
            nav: NavSettings::default(),
            behavior: BehaviorSettings::default(),
            display: DisplaySettings::default(),
            theme: ThemeSettings::default(),
            console_mode: ConsoleModeSettings::default(),
            categories: default_categories(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CategoryEntry {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub name: String,
    #[serde(default)]
    pub icon: String,
    pub kind: String,
    #[serde(default)]
    pub value: String,
}

fn default_true() -> bool { true }

fn default_categories() -> Vec<CategoryEntry> {
    vec![
        CategoryEntry { enabled: true, name: "All Games".into(), icon: "sports_esports".into(), kind: "all".into(), value: String::new() },
        CategoryEntry { enabled: true, name: "Favourites".into(), icon: "star".into(), kind: "favourite".into(), value: String::new() },
        CategoryEntry { enabled: true, name: "Recent".into(), icon: "schedule".into(), kind: "recent".into(), value: String::new() },
        CategoryEntry { enabled: true, name: "Wine".into(), icon: "wine_bar".into(), kind: "runner".into(), value: "wine".into() },
        CategoryEntry { enabled: true, name: "Native".into(), icon: "terminal".into(), kind: "runner".into(), value: "native".into() },
    ]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LibrarySettings {
    // effective range 0.6..1.5, qml slider clamps
    pub card_zoom: f64,
    pub card_spacing: i32,
    pub card_elevation: bool,
    // unload store tabs 15s after navigating away, freeing card delegates and decoded banner. even tho this seems useless. the memory usage is the same even after unloading. dunno why. fuck you qml
    pub unload_store_pages: bool,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            card_zoom: 1.0,
            card_spacing: 16,
            card_elevation: true,
            unload_store_pages: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TabsSettings {
    pub show_gachas: bool,
    pub show_epic: bool,
    pub show_gog: bool,
    pub show_steam: bool,
}

impl Default for TabsSettings {
    fn default() -> Self {
        Self { show_gachas: true, show_epic: true, show_gog: true, show_steam: true }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct NavSettings {
    pub width: i32,
    pub collapsed: bool,
}

impl Default for NavSettings {
    fn default() -> Self {
        Self { width: 180, collapsed: false }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct BehaviorSettings {
    pub minimize_on_launch: bool,
    pub save_game_logs: bool,
    pub auto_check_epic_updates_on_launch: bool,
    pub auto_check_gog_updates_on_launch: bool,
    pub auto_check_updates_on_boot: bool,
    pub show_tray_icon: bool,
    pub discord_rpc: bool,
    pub double_click_launches: bool,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplaySettings {
    // browser-style ui zoom applied via a root Scale transform in Main.qml so every
    // padding/icon/font scales uniformly. bridge clamps this; Ctrl+/- steps by 0.1.
    // this sucks not gonna lie the scaling is so ass it blurs everything
    pub scale: f64,
    pub muted_icons: bool,
    pub card_flow: String,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self { scale: 1.0, muted_icons: false, card_flow: "center".into() }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ThemeSettings {
    pub follow_system_colors: bool,
    pub follow_system_font: bool,
    pub font_family: String,
    pub colors: BTreeMap<String, String>,
    pub fill_fields: bool,
    pub radius_scale: f64,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            follow_system_colors: true,
            follow_system_font: true,
            font_family: String::new(),
            colors: BTreeMap::new(),
            fill_fields: true,
            radius_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsoleModeSettings {
    pub background: String,
    pub active: bool,
}

impl Default for ConsoleModeSettings {
    fn default() -> Self {
        Self { background: "wave".into(), active: false }
    }
}

pub fn ui_settings_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omikuji")
        .join("ui.toml")
}

impl UiSettings {
    pub fn load() -> Self {
        let path = ui_settings_path();
        if !path.exists() {
            let defaults = Self::default();
            if let Err(e) = defaults.save() {
                tracing::warn!("couldn't write defaults to {}: {} - running in-memory only", path.display(), e);
            }
            return defaults;
        }

        match std::fs::read_to_string(&path) {
            Ok(body) => toml::from_str::<UiSettings>(&body).unwrap_or_else(|e| {
                tracing::warn!("couldn't parse {}: {} - using defaults", path.display(), e);
                Self::default()
            }),
            Err(e) => {
                tracing::warn!("couldn't read {}: {} - using defaults", path.display(), e);
                Self::default()
            }
        }
    }

    pub fn set_console_mode_active(active: bool) {
        let mut settings = Self::load();
        settings.console_mode.active = active;
        let _ = settings.save();
    }

    // atomic write (tmp + rename) so a crash mid-save cant leave a half-written file (hopefully)
    pub fn save(&self) -> std::io::Result<()> {
        let path = ui_settings_path();
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
