use crate::defaults::{CopyMode, Defaults, FIELDS};
use crate::dll_packs;
use crate::fs_util::write_atomic;
use crate::media::slugify;
use crate::migration;
use crate::string_enum::string_enum;
use anyhow::{Context, Result, anyhow};
use fs_err as fs;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use toml::{Table, Value, de, ser};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Game {
    #[serde(flatten)]
    pub metadata: Metadata,
    #[serde(default)]
    pub runner: RunnerConfig,
    #[serde(default)]
    pub wine: WineConfig,
    #[serde(default)]
    pub launch: LaunchConfig,
    #[serde(default)]
    pub graphics: GraphicsConfig,
    #[serde(default)]
    pub system: SystemConfig,
    // kept at the bottom so users arent tempted to touch it; drives store
    // detection (epic/steam/gog) and shouldnt be edited by hand. touching this = boom
    #[serde(default)]
    pub source: SourceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sort_name: String,
    #[serde(default)]
    pub slug: String,
    pub exe: PathBuf,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub playtime: f64,
    #[serde(default)]
    pub last_played: String,
    #[serde(default)]
    pub added: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_pos: Option<u32>,
    #[serde(default)]
    pub banner: String,
    #[serde(default)]
    pub coverart: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub favourite: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SourceKind {
    #[default]
    Manual,
    Epic,
    Steam,
    Gog,
    Nile,
    Gacha,
}

string_enum!(SourceKind {
    Manual => "",
    Epic => "epic",
    Steam => "steam",
    Gog => "gog",
    Nile => "nile",
    Gacha => "gacha",
});

impl SourceKind {
    // the STORE value umu wants, so protonfixes can match the game
    pub fn umu_store(self) -> Option<&'static str> {
        match self {
            Self::Epic => Some("egs"),
            Self::Gog => Some("gog"),
            Self::Nile => Some("amazon"),
            _ => None,
        }
    }

    pub fn has_updates(self) -> bool {
        matches!(self, Self::Gacha | Self::Epic | Self::Gog | Self::Nile)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SourceConfig {
    #[serde(default)]
    pub kind: SourceKind,
    #[serde(default)]
    pub app_id: String,
    // eos overlay installs into the prefix and enables per-prefix; only relevant when kind == "epic"
    #[serde(default)]
    pub eos_overlay: bool,
    // auto-syncs cloud saves via legendary before launch (download) and after exit (upload) [apparently 1 game on 7 trilion has actual cloud saves on epic games]
    #[serde(default)]
    pub cloud_saves: bool,
    // populated on first cloud_saves toggle via `legendary sync-saves --accept-path`
    #[serde(default)]
    pub save_path: String,
    // patch wrapper at launch, set at import time.
    #[serde(default)]
    pub patch: String,
    // store dlc ids the user installed; updates must re-send these or gogdl drops the files apparently? i genuinely dont fully know
    #[serde(default)]
    pub dlcs: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub predownload_dismissed: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RunnerType {
    #[default]
    Wine,
    Steam,
    Flatpak,
    Native,
}

string_enum!(RunnerType {
    Wine => "wine",
    Steam => "steam",
    Flatpak => "flatpak",
    Native => "native",
});

impl RunnerType {
    pub fn is_steam(self) -> bool {
        self == Self::Steam
    }

    pub fn on_host(self) -> bool {
        matches!(self, Self::Native | Self::Flatpak)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RunnerConfig {
    #[serde(alias = "runner_type", rename = "type", default)]
    pub runner_type: RunnerType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WineConfig {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default = "default_prefix_arch")]
    pub prefix_arch: String,
    #[serde(default = "default_true")]
    pub esync: bool,
    #[serde(default = "default_true")]
    pub fsync: bool,
    #[serde(default)]
    pub ntsync: bool,
    #[serde(default = "default_true")]
    pub dxvk: bool,
    #[serde(default = "default_builtin")]
    pub dxvk_version: String,
    #[serde(default = "default_true")]
    pub vkd3d: bool,
    #[serde(default = "default_builtin")]
    pub vkd3d_version: String,
    #[serde(default = "default_true")]
    pub dxvk_nvapi: bool,
    #[serde(default = "default_builtin")]
    pub dxvk_nvapi_version: String,
    #[serde(default)]
    pub fsr: bool,
    #[serde(default)]
    pub battleye: bool,
    #[serde(default)]
    pub easyanticheat: bool,
    #[serde(default)]
    pub dpi_scaling: bool,
    #[serde(default = "default_dpi")]
    pub dpi: u32,
    #[serde(default)]
    pub dll_overrides: IndexMap<String, String>,
    #[serde(default)]
    pub dll_override_sets: Vec<String>,
    #[serde(default)]
    pub audio_driver: String,
    #[serde(default)]
    pub graphics_driver: String,
}

fn default_prefix_arch() -> String {
    "win64".to_string()
}
fn default_true() -> bool {
    true
}
fn default_builtin() -> String {
    dll_packs::BUILTIN.to_string()
}
fn default_dpi() -> u32 {
    96
}

impl Default for WineConfig {
    fn default() -> Self {
        Self {
            version: String::new(),
            prefix: String::new(),
            prefix_arch: default_prefix_arch(),
            esync: true,
            fsync: true,
            ntsync: true,
            dxvk: true,
            dxvk_version: default_builtin(),
            vkd3d: true,
            vkd3d_version: default_builtin(),
            dxvk_nvapi: true,
            dxvk_nvapi_version: default_builtin(),
            fsr: false,
            battleye: false,
            easyanticheat: false,
            dpi_scaling: false,
            dpi: default_dpi(),
            dll_overrides: IndexMap::new(),
            dll_override_sets: Vec::new(),
            audio_driver: String::new(),
            graphics_driver: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CompanionWhen {
    Before,
    #[default]
    After,
}

string_enum!(CompanionWhen {
    Before => "before",
    After => "after",
});

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct LaunchConfig {
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: String,
    #[serde(default)]
    pub command_prefix: String,
    #[serde(default)]
    pub pre_launch_script: String,
    #[serde(default)]
    pub post_exit_script: String,
    #[serde(default)]
    pub companion: String,
    #[serde(default)]
    pub companion_args: Vec<String>,
    #[serde(default)]
    pub companion_when: CompanionWhen,
    #[serde(default)]
    pub companion_delay: u32,
    #[serde(default)]
    pub env: IndexMap<String, String>,
    #[serde(default)]
    pub env_sets: Vec<String>,
}

impl LaunchConfig {
    pub fn prune_companion(&mut self) {
        if self.companion.trim().is_empty() {
            self.companion.clear();
            self.companion_args.clear();
            self.companion_when = CompanionWhen::default();
            self.companion_delay = 0;
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GraphicsConfig {
    #[serde(default)]
    pub mangohud: bool,
    #[serde(default)]
    pub gpu: String,
    #[serde(default)]
    pub gamescope: GamescopeConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GamescopeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub game_width: u32,
    #[serde(default)]
    pub game_height: u32,
    #[serde(default)]
    pub fps: u32,
    #[serde(default)]
    pub refresh_rate: u32,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub borderless: bool,
    #[serde(default)]
    pub integer_scaling: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default)]
    pub filter: String,
    #[serde(default)]
    pub fsr_sharpness: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SystemConfig {
    #[serde(default)]
    pub gamemode: bool,
    #[serde(default)]
    pub prevent_sleep: bool,
    #[serde(default)]
    pub pulse_latency: bool,
    #[serde(default)]
    pub cpu_limit: u32,
    #[serde(default)]
    pub discord_rpc: bool,
}

fn default_color() -> String {
    "#1a1a2e".to_string()
}

fn toml_error_summary(err: &de::Error, contents: &str) -> String {
    let message = err.message();
    let Some(span) = err.span() else {
        return err.to_string().trim_end().replace("\nin ", " in ");
    };
    let before = &contents[..span.start];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let line = contents[line_start..].lines().next().unwrap_or_default();
    match line.split_once('=') {
        Some((key, _)) => format!("{message} in `{}`", key.trim()),
        None => format!("line {}: {message}", before.matches('\n').count() + 1),
    }
}

fn rfc3339_of(t: std::time::SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn rfc3339_now() -> String {
    rfc3339_of(std::time::SystemTime::now())
}

impl Metadata {
    pub fn new(id: String, name: String, exe: PathBuf) -> Self {
        Self {
            id,
            name,
            sort_name: String::new(),
            slug: String::new(),
            exe,
            color: default_color(),
            playtime: 0.0,
            last_played: String::new(),
            added: rfc3339_now(),
            custom_pos: None,
            banner: String::new(),
            coverart: String::new(),
            icon: String::new(),
            favourite: false,
            hidden: false,
            categories: Vec::new(),
        }
    }
    pub fn slug(&self) -> String {
        if self.slug.trim().is_empty() {
            slugify(&self.name)
        } else {
            self.slug.clone()
        }
    }
}

#[derive(Debug, Default)]
pub struct Library {
    pub game: Vec<Game>,
    pub load_errors: Vec<String>,
}

impl Library {
    pub fn game(&self, id: &str) -> Option<&Game> {
        self.game.iter().find(|g| g.id() == id)
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.game.iter().position(|g| g.id() == id)
    }

    fn game_files() -> Result<Vec<PathBuf>> {
        let dir = crate::library_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if path.extension().and_then(|s| s.to_str()) == Some("toml") && !name.starts_with('.') {
                files.push(path);
            }
        }
        Ok(files)
    }

    pub fn game_ids_by_app_id(kind: SourceKind) -> HashMap<String, String> {
        Self::game_files()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .filter_map(|content| toml::from_str::<Game>(&content).ok())
            .filter(|game| game.source.kind == kind && !game.source.app_id.is_empty())
            .map(|game| (game.source.app_id, game.metadata.id))
            .collect()
    }

    pub fn load() -> Result<Self> {
        let mut games = Vec::new();
        let mut load_errors = Vec::new();

        for path in Self::game_files()? {
            match Self::load_game(&path) {
                Ok(mut game) => {
                    if game.metadata.added.is_empty() {
                        let t = fs::metadata(&path)
                            .ok()
                            .and_then(|m| m.created().or_else(|_| m.modified()).ok())
                            .unwrap_or(std::time::UNIX_EPOCH);
                        game.metadata.added = rfc3339_of(t);
                        if let Ok(contents) = Self::game_toml(&game) {
                            let _ = write_atomic(&path, contents);
                        }
                    }
                    games.push(game)
                }
                Err(e) => {
                    tracing::warn!("failed to load game: {e:#}");
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    load_errors.push(format!("{name}: {}", e.root_cause()));
                }
            }
        }

        games.sort_by(|a, b| a.added_key().cmp(&b.added_key()));

        Ok(Self {
            game: games,
            load_errors,
        })
    }

    fn load_game(path: &Path) -> Result<Game> {
        let contents = fs::read_to_string(path)?;
        let mut table: Table = toml::from_str(&contents)
            .map_err(|e| anyhow!(toml_error_summary(&e, &contents)))
            .with_context(|| format!("parsing {}", path.display()))?;
        let renamed = migration::rename_alongside_keys(&mut table);
        let game: Game = Value::Table(table)
            .try_into()
            .map_err(|e| {
                let e = toml::from_str::<Metadata>(&contents).err().unwrap_or(e);
                anyhow!(toml_error_summary(&e, &contents))
            })
            .with_context(|| format!("parsing {}", path.display()))?;
        if renamed {
            write_atomic(path, Self::game_toml(&game)?)?;
        }
        Ok(game)
    }

    pub fn load_game_by_id(id: &str) -> Result<Option<Game>> {
        Self::find_game_file_by_id(id)?
            .map(|path| Self::load_game(&path))
            .transpose()
    }

    fn game_toml(game: &Game) -> Result<String, ser::Error> {
        let contents = toml::to_string_pretty(game)?;
        Ok(contents.replace(
            "\n[source]\n",
            "\n# this section is used by omikuji to identify a game and behave accordingly, please do not touch unless you know why\n[source]\n",
        ))
    }

    pub fn save_game(game: &Game) -> Result<()> {
        let dir = crate::library_dir();
        fs::create_dir_all(&dir)?;

        // renames keep the old filename
        let path = match Self::find_game_file_by_id(&game.metadata.id)? {
            Some(existing_path) => existing_path,
            None => {
                let filename = if game.runner.runner_type.is_steam() {
                    format!("steam_{}.toml", game.metadata.id)
                } else {
                    format!("{}_{}.toml", slugify(&game.metadata.name), game.metadata.id)
                };
                dir.join(filename)
            }
        };

        let contents = Self::game_toml(game)?;
        write_atomic(&path, contents).with_context(|| format!("writing {}", path.display()))?;

        Ok(())
    }

    fn find_game_file_by_id(id: &str) -> Result<Option<PathBuf>> {
        let suffix = format!("_{id}.toml");
        Ok(Self::game_files()?.into_iter().find(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|name| name.ends_with(&suffix))
        }))
    }

    pub fn remove_game_file(id: &str) -> Result<()> {
        if let Some(path) = Self::find_game_file_by_id(id)? {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

// names built as slug_id (old shortcut targets, log folders) carry the id after the last underscore
// also used for the logs files
pub fn id_from_slug_id(name: &str) -> &str {
    name.rsplit_once('_').map_or(name, |(_, id)| id)
}

// 6 char alphanumeric id
pub fn generate_id() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    );
    let mut hash = hasher.finish();
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let base = chars.len() as u64;
    (0..6)
        .map(|_| {
            let c = chars[(hash % base) as usize];
            hash /= base;
            c as char
        })
        .collect()
}

impl Game {
    pub fn new(name: String, exe: PathBuf) -> Self {
        Self {
            metadata: Metadata::new(generate_id(), name, exe),
            source: SourceConfig::default(),
            runner: RunnerConfig::default(),
            wine: WineConfig::default(),
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.wine.prefix = prefix.into();
        self
    }

    pub fn with_runner(mut self, runner_type: RunnerType) -> Self {
        self.runner.runner_type = runner_type;
        self
    }

    pub fn with_runner_version(mut self, version: impl Into<String>) -> Self {
        self.wine.version = version.into();
        self
    }

    pub fn with_prefix_arch(mut self, arch: impl Into<String>) -> Self {
        self.wine.prefix_arch = arch.into();
        self
    }

    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    pub fn slug(&self) -> String {
        self.metadata.slug()
    }

    pub fn slug_with_id(&self) -> String {
        format!("{}_{}", self.slug(), self.id())
    }

    pub fn added_key(&self) -> (&str, &str) {
        (&self.metadata.added, &self.metadata.id)
    }

    pub fn custom_key(&self) -> (u32, (&str, &str)) {
        (
            self.metadata.custom_pos.unwrap_or(u32::MAX),
            self.added_key(),
        )
    }

    pub fn display_sort_key(&self) -> String {
        let m = &self.metadata;
        let s = if m.sort_name.trim().is_empty() {
            &m.name
        } else {
            &m.sort_name
        };
        s.trim().to_lowercase()
    }

    // falls back to metadata.id for games impoted before the source section existed
    pub fn effective_app_id(&self) -> &str {
        if self.source.app_id.is_empty() {
            &self.metadata.id
        } else {
            &self.source.app_id
        }
    }

    // epic games are launched via legendary, not wine directly, i mean still wine but through legendary
    pub fn is_epic(&self) -> bool {
        self.source.kind == SourceKind::Epic
    }

    // steam/flatpak/native launch outside wine, so they don't use an omikuji-managed prefix, damn gaijin...
    pub fn uses_wine_prefix(&self) -> bool {
        self.runner.runner_type == RunnerType::Wine
    }

    pub fn launches_exe(&self) -> bool {
        match self.runner.runner_type {
            RunnerType::Native => true,
            RunnerType::Wine => !self.is_epic(),
            RunnerType::Steam | RunnerType::Flatpak => false,
        }
    }

    // skips fields the caller already set so per-source picks (steam:appid etc) survive
    pub fn seed_from_defaults(&mut self, d: &Defaults) {
        d.apply_to(self, FIELDS.iter().map(|f| f.key), CopyMode::Seed);
    }
}
