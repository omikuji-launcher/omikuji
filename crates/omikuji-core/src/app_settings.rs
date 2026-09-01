// app preferences. live-mutable, app-written (via the bridge).
//
// separate from settings.toml on purpose: settings.toml is hacker knobs (paths, remote urls)
// that the user edits and the app reads once at startup. app.toml is the app's own scratch
// for zoom/tabs/layout/behaviour prefs; app writes, qml reads live through the bridge, no restart needed.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AppSettings {
    pub language: String,
    #[serde(default, rename = "last_seen_changelog", skip_serializing)]
    pub legacy_last_seen_changelog: String,
    pub library: LibrarySettings,
    pub display: DisplaySettings,
    pub theme: ThemeSettings,
    pub nav: NavSettings,
    pub tabs: TabsSettings,
    #[serde(default)]
    pub console_mode: ConsoleModeSettings,
    #[serde(default = "default_categories")]
    pub categories: Vec<CategoryEntry>,
    #[serde(default)]
    pub dialog_sizes: BTreeMap<String, [f64; 2]>,
    pub behavior: BehaviorSettings,
    #[serde(default)]
    pub download: DownloadSettings,
    #[serde(default)]
    pub env_sets: Vec<KvSet>,
    #[serde(default)]
    pub dll_sets: Vec<KvSet>,
    #[serde(default)]
    pub template_vars: BTreeMap<String, String>,
    #[serde(default)]
    pub state: AppState,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct AppState {
    pub last_seen_changelog: String,
    pub welcome_seen: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "system".into(),
            legacy_last_seen_changelog: String::new(),
            library: LibrarySettings::default(),
            display: DisplaySettings::default(),
            theme: ThemeSettings::default(),
            nav: NavSettings::default(),
            tabs: TabsSettings::default(),
            console_mode: ConsoleModeSettings::default(),
            categories: default_categories(),
            dialog_sizes: BTreeMap::new(),
            behavior: BehaviorSettings::default(),
            download: DownloadSettings::default(),
            env_sets: Vec::new(),
            dll_sets: Vec::new(),
            template_vars: BTreeMap::new(),
            state: AppState::default(),
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
    #[serde(default)]
    pub auto_name: Option<bool>,
}

fn default_true() -> bool {
    true
}

fn default_categories() -> Vec<CategoryEntry> {
    vec![
        CategoryEntry {
            enabled: true,
            name: "All Games".into(),
            icon: "sports_esports".into(),
            kind: "all".into(),
            value: String::new(),
            auto_name: Some(true),
        },
        CategoryEntry {
            enabled: true,
            name: "Favourites".into(),
            icon: "star".into(),
            kind: "favourite".into(),
            value: String::new(),
            auto_name: Some(true),
        },
        CategoryEntry {
            enabled: true,
            name: "Recent".into(),
            icon: "schedule".into(),
            kind: "recent".into(),
            value: String::new(),
            auto_name: Some(true),
        },
        CategoryEntry {
            enabled: true,
            name: "Wine".into(),
            icon: "wine_bar".into(),
            kind: "runner".into(),
            value: "wine".into(),
            auto_name: Some(true),
        },
        CategoryEntry {
            enabled: true,
            name: "Native".into(),
            icon: "terminal".into(),
            kind: "runner".into(),
            value: "native".into(),
            auto_name: Some(true),
        },
    ]
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KvPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KvSet {
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub vars: Vec<KvPair>,
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
        Self {
            show_gachas: true,
            show_epic: true,
            show_gog: true,
            show_steam: true,
        }
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
        Self {
            width: 180,
            collapsed: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BehaviorSettings {
    pub minimize_on_launch: bool,
    pub save_game_logs: bool,
    pub auto_check_epic_updates_on_launch: bool,
    pub auto_check_gog_updates_on_launch: bool,
    pub auto_check_updates_on_boot: bool,
    pub show_tray_icon: bool,
    pub double_click_launches: bool,
    pub discord_show_launcher: bool,
    pub notify_on_download_complete: bool,
    pub ignore_steam_runners: bool,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            minimize_on_launch: false,
            save_game_logs: false,
            auto_check_epic_updates_on_launch: false,
            auto_check_gog_updates_on_launch: false,
            auto_check_updates_on_boot: false,
            show_tray_icon: false,
            double_click_launches: false,
            discord_show_launcher: true,
            notify_on_download_complete: true,
            ignore_steam_runners: false,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct DownloadSettings {
    // must stay above the sub-tables, toml puts bare keys before them
    pub bandwidth_mb_per_sec: f64,
    pub epic: EpicDownloadSettings,
    pub gog: GogDownloadSettings,
    pub gacha: GachaDownloadSettings,
}
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct EpicDownloadSettings {
    pub workers: i32,
    pub shared_memory_mb: i32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct GogDownloadSettings {
    pub workers: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GachaDownloadSettings {
    pub max_connections: i32,
    pub patch_threads: i32,
}

impl Default for GachaDownloadSettings {
    fn default() -> Self {
        Self {
            max_connections: 32,
            patch_threads: 4,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LogRule {
    pub pattern: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DisplaySettings {
    // browser-style ui zoom applied via a root Scale transform in Main.qml so every
    // padding/icon/font scales uniformly. bridge clamps this; Ctrl+/- steps by 0.1.
    // this sucks not gonna lie the scaling is so ass it blurs everything
    pub scale: f64,
    pub muted_icons: bool,
    pub filled_icons: bool,
    pub show_hidden: bool,
    pub dim_hidden: bool,
    pub show_steam_prefixes: bool,
    pub card_flow: String,
    pub card_sort: String,
    pub card_style: String,
    pub highlight_logs: bool,
    pub log_rules: Vec<LogRule>,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            muted_icons: false,
            filled_icons: false,
            show_hidden: false,
            dim_hidden: false,
            show_steam_prefixes: false,
            card_flow: "center".into(),
            card_sort: "default".into(),
            card_style: "normal".into(),
            highlight_logs: true,
            log_rules: Vec::new(),
        }
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
    pub fonts: BTreeMap<String, u32>,
    pub radii: BTreeMap<String, u32>,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            follow_system_colors: true,
            follow_system_font: true,
            font_family: String::new(),
            colors: BTreeMap::new(),
            fill_fields: true,
            fonts: BTreeMap::new(),
            radii: BTreeMap::new(),
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
        Self {
            background: "wave".into(),
            active: false,
        }
    }
}

fn data_root() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omikuji")
}

// TODO: remove the ui.toml handling after a 1 or 2 releases ig
pub fn app_settings_path() -> PathBuf {
    let path = data_root().join("app.toml");
    let legacy = data_root().join("ui.toml");
    if path.exists() || !legacy.exists() {
        return path;
    }
    match std::fs::rename(&legacy, &path) {
        Ok(()) => path,
        Err(e) => {
            tracing::warn!(
                "couldn't rename {} to {}: {} - reading the legacy path",
                legacy.display(),
                path.display(),
                e
            );
            legacy
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let path = app_settings_path();
        if !path.exists() {
            let defaults = Self::default();
            if let Err(e) = defaults.save() {
                tracing::warn!(
                    "couldn't write defaults to {}: {} - running in-memory only",
                    path.display(),
                    e
                );
            }
            return defaults;
        }

        match std::fs::read_to_string(&path) {
            Ok(body) => match toml::from_str::<AppSettings>(&body) {
                Ok(mut settings) => {
                    let moved = settings.migrate_legacy_app_state();
                    if settings.migrate_category_auto_names() || moved {
                        let _ = settings.save();
                    }
                    settings
                }
                Err(e) => {
                    tracing::warn!("couldn't parse {}: {} - using defaults", path.display(), e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!("couldn't read {}: {} - using defaults", path.display(), e);
                Self::default()
            }
        }
    }

    // TODO: remove after a 1 or 2 releases ig
    fn migrate_legacy_app_state(&mut self) -> bool {
        if self.legacy_last_seen_changelog.is_empty() {
            return false;
        }
        if self.state.last_seen_changelog.is_empty() {
            self.state.last_seen_changelog = std::mem::take(&mut self.legacy_last_seen_changelog);
        } else {
            self.legacy_last_seen_changelog.clear();
        }
        true
    }

    // TODO: remove after a 1 or 2 releases ig
    fn migrate_category_auto_names(&mut self) -> bool {
        let defaults = default_categories();
        let mut changed = false;
        for cat in self.categories.iter_mut() {
            if cat.auto_name.is_some() {
                continue;
            }
            cat.auto_name = Some(
                defaults
                    .iter()
                    .any(|d| d.kind == cat.kind && d.value == cat.value && d.name == cat.name),
            );
            changed = true;
        }
        changed
    }

    pub fn set_console_mode_active(active: bool) {
        let mut settings = Self::load();
        settings.console_mode.active = active;
        let _ = settings.save();
    }

    // atomic write (tmp + rename) so a crash mid-save cant leave a half-written file (hopefully)
    pub fn save(&self) -> std::io::Result<()> {
        let body = toml::to_string_pretty(self)
            .map(|s| {
                s.replace(
                    "\n[state]\n",
                    "\n# this section is omikuji's own bookkeeping for how it should behave, not meant to be edited by hand\n[state]\n",
                )
            })
            .map_err(std::io::Error::other)?;
        crate::fs_util::write_atomic(&app_settings_path(), body)
    }
}
