use crate::media::slugify;
use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SourceConfig {
    #[serde(default)]
    pub kind: String,
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
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RunnerType {
    #[default]
    Wine,
    Steam,
    Flatpak,
    Native,
}

impl RunnerType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wine => "wine",
            Self::Steam => "steam",
            Self::Flatpak => "flatpak",
            Self::Native => "native",
        }
    }

    pub fn is_steam(self) -> bool {
        self == Self::Steam
    }

    pub fn on_host(self) -> bool {
        matches!(self, Self::Native | Self::Flatpak)
    }
}

impl std::str::FromStr for RunnerType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "steam" => Ok(Self::Steam),
            "flatpak" => Ok(Self::Flatpak),
            "native" => Ok(Self::Native),
            _ => Err(()),
        }
    }
}

impl Serialize for RunnerType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

// a typo in the toml falls back to wine instead of failing the whole game load
impl<'de> Deserialize<'de> for RunnerType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(String::deserialize(d)?.parse().unwrap_or_default())
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
    crate::dll_packs::BUILTIN.to_string()
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
            ntsync: false,
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
            dpi: 96,
            dll_overrides: IndexMap::new(),
            dll_override_sets: Vec::new(),
            audio_driver: String::new(),
            graphics_driver: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AlongsideWhen {
    Before,
    #[default]
    After,
}

impl AlongsideWhen {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

impl std::str::FromStr for AlongsideWhen {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "before" => Ok(Self::Before),
            "after" => Ok(Self::After),
            _ => Err(()),
        }
    }
}

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
    pub alongside: String,
    #[serde(default)]
    pub alongside_args: Vec<String>,
    #[serde(default)]
    pub alongside_when: AlongsideWhen,
    #[serde(default)]
    pub alongside_delay: u32,
    #[serde(default)]
    pub env: IndexMap<String, String>,
    #[serde(default)]
    pub env_sets: Vec<String>,
}

impl LaunchConfig {
    pub fn prune_alongside(&mut self) {
        if self.alongside.trim().is_empty() {
            self.alongside.clear();
            self.alongside_args.clear();
            self.alongside_when = AlongsideWhen::default();
            self.alongside_delay = 0;
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

pub fn default_color() -> String {
    "#1a1a2e".to_string()
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
            crate::media::slugify(&self.name)
        } else {
            self.slug.clone()
        }
    }
}

#[derive(Debug, Default)]
pub struct Library {
    pub game: Vec<Game>,
}

impl Library {
    pub fn library_dir() -> PathBuf {
        crate::library_dir()
    }

    pub fn game_ids_by_app_id(kind: &str) -> HashMap<String, String> {
        let mut out = HashMap::new();
        let dir = Self::library_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(game) = toml::from_str::<Game>(&content) else {
                continue;
            };
            if game.source.kind == kind && !game.source.app_id.is_empty() {
                out.insert(game.source.app_id, game.metadata.id);
            }
        }
        out
    }

    pub fn load() -> Result<Self> {
        let dir = Self::library_dir();
        if !dir.exists() {
            return Ok(Self::default());
        }

        let mut games = Vec::new();

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') || name.ends_with('~') {
                continue;
            }

            match Self::load_game(&path) {
                Ok(mut game) => {
                    if game.metadata.added.is_empty() {
                        let t = fs::metadata(&path)
                            .ok()
                            .and_then(|m| m.created().or_else(|_| m.modified()).ok())
                            .unwrap_or(std::time::UNIX_EPOCH);
                        game.metadata.added = rfc3339_of(t);
                        if let Ok(contents) = Self::game_toml(&game) {
                            let _ = crate::fs_util::write_atomic(&path, contents);
                        }
                    }
                    games.push(game)
                }
                Err(e) => tracing::warn!("failed to load game {}: {}", path.display(), e),
            }
        }

        games.sort_by(|a, b| a.added_key().cmp(&b.added_key()));

        Ok(Self { game: games })
    }

    fn load_game(path: &PathBuf) -> Result<Game> {
        let contents =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let game: Game =
            toml::from_str(&contents).with_context(|| format!("parsing {}", path.display()))?;
        Ok(game)
    }

    pub fn load_game_by_id(id: &str) -> Result<Option<Game>> {
        let dir = Self::library_dir();
        if !dir.exists() {
            return Ok(None);
        }

        let suffix = format!("_{}.toml", id);
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.ends_with(&suffix) {
                return Self::load_game(&path).map(Some);
            }
        }
        Ok(None)
    }

    pub fn save_game(&self, game: &Game) -> Result<()> {
        Self::save_game_static(game)
    }

    fn game_toml(game: &Game) -> Result<String, toml::ser::Error> {
        let contents = toml::to_string_pretty(game)?;
        Ok(contents.replace(
            "\n[source]\n",
            "\n# this section is used by omikuji to identify a game and behave accordingly, please do not touch unless you know why\n[source]\n",
        ))
    }

    pub fn save_game_static(game: &Game) -> Result<()> {
        let dir = Self::library_dir();
        fs::create_dir_all(&dir)?;

        // reuse exsiting filename if found by id, so renames dont create new files and
        // leave the old one orphaned. steam games use "steam_{appid}.toml" format
        let path = match Self::find_game_file_by_id(&game.metadata.id) {
            Ok(Some(existing_path)) => existing_path,
            _ => {
                let filename = if game.runner.runner_type.is_steam() {
                    format!("steam_{}.toml", game.metadata.id)
                } else {
                    format!("{}_{}.toml", slugify(&game.metadata.name), game.metadata.id)
                };
                dir.join(filename)
            }
        };

        let contents = Self::game_toml(game)?;
        crate::fs_util::write_atomic(&path, contents)
            .with_context(|| format!("writing {}", path.display()))?;

        Ok(())
    }

    fn find_game_file_by_id(id: &str) -> Result<Option<PathBuf>> {
        let dir = Self::library_dir();
        if !dir.exists() {
            return Ok(None);
        }

        let suffix = format!("_{}.toml", id);
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.ends_with(&suffix) {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub fn remove_game_file(id: &str) -> Result<()> {
        if let Some(path) = Self::find_game_file_by_id(id)? {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
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
    let hash = hasher.finish();
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    (0..6)
        .map(|i| chars[((hash >> (i * 6)) & 0x1F) as usize % chars.len()] as char)
        .collect()
}

impl Game {
    pub fn new(name: String, exe: PathBuf) -> Self {
        Self::with_options(name, exe, None, Some(RunnerType::Wine), None)
    }

    pub fn with_options(
        name: String,
        exe: PathBuf,
        prefix: Option<String>,
        runner_type: Option<RunnerType>,
        runner_version: Option<String>,
    ) -> Self {
        Self {
            metadata: Metadata::new(generate_id(), name, exe),
            source: SourceConfig::default(),
            runner: RunnerConfig {
                runner_type: runner_type.unwrap_or_default(),
            },
            wine: WineConfig {
                version: runner_version.unwrap_or_default(),
                prefix: prefix.unwrap_or_default(),
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        }
    }

    pub fn id(&self) -> &str {
        &self.metadata.id
    }
    pub fn name(&self) -> &str {
        &self.metadata.name
    }
    pub fn exe(&self) -> &PathBuf {
        &self.metadata.exe
    }

    pub fn slug(&self) -> String {
        self.metadata.slug()
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

    // epic games are launched via legendary, not wine directly, i mean still wine but through legendary
    pub fn is_epic(&self) -> bool {
        self.source.kind == "epic"
    }

    // steam/flatpak/native launch outside wine, so they don't use an omikuji-managed prefix, damn gaijin...
    pub fn uses_wine_prefix(&self) -> bool {
        self.runner.runner_type == RunnerType::Wine
    }

    // skips fields the caller already set so per-source picks (steam:appid etc) survive
    pub fn seed_from_defaults(&mut self, d: &crate::defaults::Defaults) {
        use crate::defaults::{CopyMode, FIELDS};
        d.apply_to(self, FIELDS.iter().map(|f| f.key), CopyMode::Seed);
    }
}
