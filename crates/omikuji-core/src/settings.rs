// launcher settings. loaded once at startup into a OnceLock singleton.
//
// the settings file's own location is a fixed anchor at
// dirs::data_dir()/omikuji/settings.toml; not user-redirectable, else chicken-and-egg when resloving where to read the redirect from.
// everything it points at *is* user-redirectable via [paths].
//
// ui preferences (zoom, theme, tab visibility) live in ui_settings.rs,
// different lifecycle, different audience. different mind. alpha or beta. your choice mate.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub paths: PathsSettings,
    pub assets: AssetsSettings,
    pub components: ComponentsSettings,
    pub steam: SteamSettings,
}

// empty api_key disables remote playtime sync; local process-tracking still works
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct SteamSettings {
    pub api_key: String,
}

// paths are stored as strings so a leading `~` survives TOML round-trips;
// accessors shellexpand::tilde on read.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PathsSettings {
    pub data_dir: String,
    pub library_dir: String,
    pub gachas_dir: String,
    pub components_dir: String,
    pub runners_dir: String,
    pub layers_dir: String,
    pub prefixes_dir: String,
    pub cache_dir: String,
    pub logs_dir: String,
    pub runtime_dir: String,
}

impl Default for PathsSettings {
    fn default() -> Self {
        let base = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("omikuji");
        let s = |sub: &str| base.join(sub).to_string_lossy().into_owned();
        Self {
            data_dir: base.to_string_lossy().into_owned(),
            library_dir: s("library"),
            gachas_dir: s("gachas"),
            components_dir: s("components"),
            runners_dir: String::new(),
            layers_dir: String::new(),
            prefixes_dir: s("prefixes"),
            cache_dir: s("cache"),
            logs_dir: s("logs"),
            runtime_dir: s("runtime"),
        }
    }
}

// single source of truth for the assets repo; fetcher appends paths like
// `gacha/{pub}/{game}/manifest.json`. repointing to a fork is a one-line edit
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AssetsSettings {
    pub fetch_url: String,
}

impl Default for AssetsSettings {
    fn default() -> Self {
        Self {
            fetch_url: "https://raw.githubusercontent.com/reakjra/omikuji-assets/main"
                .into(),
        }
    }
}

// clearing a component field breaks that component's install; theres no compile-time fallback
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ComponentsSettings {
    pub umu_run: String,
    pub hpatchz: String,
    pub legendary: String,
    pub gogdl: String,
    pub jadeite: String,
    pub egl_dummy: String,
}

impl Default for ComponentsSettings {
    fn default() -> Self {
        Self {
            umu_run: "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest".into(),
            hpatchz: "https://api.github.com/repos/sisong/HDiffPatch/releases/latest".into(),
            legendary: "https://api.github.com/repos/derrod/legendary/releases/latest".into(),
            gogdl: "https://api.github.com/repos/Heroic-Games-Launcher/heroic-gogdl/releases/latest".into(), // why does gogdl feels like a gurgle. goGLdl
            jadeite: "https://codeberg.org/api/v1/repos/mkrsym1/jadeite/releases/latest".into(), // might want to ship it as non-default once day. one day 
            egl_dummy: "https://raw.githubusercontent.com/reakjra/omikuji-assets/main/runtime/epic/EpicGamesLauncher.exe".into(),
        }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

// fixed anchor; uses dirs::data_dir() directly, NOT our own settings abstractin,
// to avoid the chicken-and-egg of resolving the file's location from itself.
pub fn settings_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omikuji")
        .join("settings.toml")
}

pub fn get() -> &'static Settings {
    SETTINGS.get_or_init(load_or_default)
}

fn load_or_default() -> Settings {
    let path = settings_path();
    if !path.exists() {
        let defaults = Settings::default();
        if let Err(e) = save(&defaults) {
            tracing::warn!("couldn't write defaults to {}: {} - running in-memory only", path.display(), e);
        }
        return defaults;
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<Settings>(&contents) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("couldn't parse {}: {} - using defaults", path.display(), e);
                Settings::default()
            }
        },
        Err(e) => {
            tracing::warn!("couldn't read {}: {} - using defaults", path.display(), e);
            Settings::default()
        }
    }
}

pub fn save(settings: &Settings) -> std::io::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = toml::to_string_pretty(settings)
        .map_err(std::io::Error::other)?;
    let header = "# omikuji settings\n\
                  # edit and restart the launcher to apply.\n\
                  # paths accept `~` (expanded to $HOME on read).\n\n";
    std::fs::write(path, format!("{}{}", header, body))
}

pub fn expand(path: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(path).into_owned())
}
