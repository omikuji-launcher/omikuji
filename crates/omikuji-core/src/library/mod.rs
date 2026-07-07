use crate::media::slugify;use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;



#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub categories: Vec<String>,
}

// drives detection (epic legendary wrapping, store-specific launch flows) and the ui badge/icon.
// orthogonal to runner: an epic game still uses runner_type="wine", source.kind="epic".
// kind values: "" (manual), "epic", "steam", "gog", "gacha"...
// honestly idk if i should use the relative epic/gog/gacha badges in the library for these games. cause for steam *steam* launches them, while installing these three latter stores you're still launching
// them on local with your own custom wine stuff, so i wonder... i wonder i wonder i wonder
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    // patch wrapper at launch. currently only "jadeite" (hsr telemetry bypass). set at import time.
    #[serde(default)]
    pub patch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct RunnerConfig {
    #[serde(alias = "runner_type", rename = "type", default)]
    pub runner_type: String, // "wine", "steam", "flatpak"
}


#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
    pub dxvk: bool,
    #[serde(default)]
    pub dxvk_version: String,
    #[serde(default)]
    pub vkd3d: bool,
    #[serde(default)]
    pub vkd3d_version: String,
    #[serde(default)]
    pub d3d_extras: bool,
    #[serde(default)]
    pub d3d_extras_version: String,
    #[serde(default)]
    pub dxvk_nvapi: bool,
    #[serde(default)]
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
    pub dll_overrides: HashMap<String, String>,
    #[serde(default)]
    pub dll_override_sets: Vec<String>,
    #[serde(default)]
    pub audio_driver: String,
    #[serde(default)]
    pub graphics_driver: String,
}

fn default_prefix_arch() -> String { "win64".to_string() }
fn default_true() -> bool { true }
fn default_dpi() -> u32 { 96 }

impl Default for WineConfig {
    fn default() -> Self {
        Self {
            version: String::new(),
            prefix: String::new(),
            prefix_arch: default_prefix_arch(),
            esync: true,
            fsync: true,
            ntsync: false,
            dxvk: false,
            dxvk_version: String::new(),
            vkd3d: false,
            vkd3d_version: String::new(),
            d3d_extras: false,
            d3d_extras_version: String::new(),
            dxvk_nvapi: false,
            dxvk_nvapi_version: String::new(),
            fsr: false,
            battleye: false,
            easyanticheat: false,
            dpi_scaling: false,
            dpi: 96,
            dll_overrides: HashMap::new(),
            dll_override_sets: Vec::new(),
            audio_driver: String::new(),
            graphics_driver: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub env_sets: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphicsConfig {
    #[serde(default)]
    pub mangohud: bool,
    #[serde(default)]
    pub gpu: String,
    #[serde(default)]
    pub gamescope: GamescopeConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemConfig {
    #[serde(default)]
    pub gamemode: bool,
    #[serde(default)]
    pub prevent_sleep: bool,
    #[serde(default)]
    pub pulse_latency: bool,
    #[serde(default)]
    pub cpu_limit: u32,
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
            categories: Vec::new(),
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

    // scan library/ for entries matching the given source kind and return their app_ids.
    // cheap-ish directory scan; callers can spawn-blocking if needed off the ui thread.
    pub fn app_ids_for_source(kind: &str) -> Vec<String> {
        let mut out = Vec::new();
        let dir = Self::library_dir();
        let Ok(entries) = std::fs::read_dir(&dir) else { return out };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let Ok(game) = toml::from_str::<Game>(&content) else { continue };
            if game.source.kind == kind && !game.source.app_id.is_empty() {
                out.push(game.source.app_id);
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
                        if let Ok(contents) = toml::to_string_pretty(&game) {
                            let _ = fs::write(&path, contents);
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
        let contents = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let game: Game = toml::from_str(&contents)
            .with_context(|| format!("parsing {}", path.display()))?;
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

    pub fn add_game(&mut self, game: Game) -> Result<()> {
        self.save_game(&game)?;
        self.game.push(game);
        Ok(())
    }

    pub fn save_game(&self, game: &Game) -> Result<()> {
        Self::save_game_static(game)
    }

    pub fn save_game_static(game: &Game) -> Result<()> {
        let dir = Self::library_dir();
        fs::create_dir_all(&dir)?;

        // reuse exsiting filename if found by id, so renames dont create new files and
        // leave the old one orphaned. steam games use "steam_{appid}.toml" format
        let path = match Self::find_game_file_by_id(&game.metadata.id) {
            Ok(Some(existing_path)) => existing_path,
            _ => {
                let filename = if game.runner.runner_type == "steam" {
                    format!("steam_{}.toml", game.metadata.id)
                } else {
                    format!("{}_{}.toml", slugify(&game.metadata.name), game.metadata.id)
                };
                dir.join(filename)
            }
        };

        let contents = toml::to_string_pretty(game)?;
        fs::write(&path, contents)
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

    pub fn remove_game(&mut self, id: &str) -> Result<bool> {
        let path = match Self::find_game_file_by_id(id) {
            Ok(Some(p)) => p,
            _ => {
                Self::library_dir().join(format!("_{}.toml", id))
            }
        };

        if path.exists() {
            let _ = fs::remove_file(&path);
        }

        if let Some(idx) = self.game.iter().position(|g| g.metadata.id == id) {
            self.game.remove(idx);
            return Ok(true);
        }
        Ok(false)
    }

    pub fn save_all(&self) -> Result<()> {
        for game in &self.game {
            self.save_game(game)?;
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
        Self::with_options(name, exe, None, Some("wine".to_string()), None)
    }

    pub fn with_options(
        name: String,
        exe: PathBuf,
        prefix: Option<String>,
        runner_type: Option<String>,
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

    pub fn id(&self) -> &str { &self.metadata.id }
    pub fn name(&self) -> &str { &self.metadata.name }
    pub fn exe(&self) -> &PathBuf { &self.metadata.exe }

    pub fn added_key(&self) -> (&str, &str) {
        (&self.metadata.added, &self.metadata.id)
    }

    pub fn custom_key(&self) -> (u32, (&str, &str)) {
        (self.metadata.custom_pos.unwrap_or(u32::MAX), self.added_key())
    }

    pub fn display_sort_key(&self) -> String {
        let m = &self.metadata;
        let s = if m.sort_name.trim().is_empty() { &m.name } else { &m.sort_name };
        s.trim().to_lowercase()
    }

    // epic games are launched via legendary, not wine directly, i mean still wine but through legendary
    pub fn is_epic(&self) -> bool {
        self.source.kind == "epic"
    }

    // steam/flatpak/native launch outside wine, so they don't use an omikuji-managed prefix, damn gaijin...
    pub fn uses_wine_prefix(&self) -> bool {
        !matches!(self.runner.runner_type.as_str(), "steam" | "flatpak" | "native")
    }

    // skips fields the caller already set so per-source picks (steam:appid etc) survive
    pub fn seed_from_defaults(&mut self, d: &crate::defaults::Defaults) {
        fn seed<T: PartialEq + Clone>(field: &mut T, stock: &T, default: &Option<T>) {
            if *field == *stock
                && let Some(v) = default { *field = v.clone(); }
        }

        let w = WineConfig::default();
        let g = GraphicsConfig::default();
        let gs_def = GamescopeConfig::default();
        let s = SystemConfig::default();
        let l = LaunchConfig::default();

        seed(&mut self.wine.version, &w.version, &d.wine.version);
        seed(&mut self.wine.prefix, &w.prefix, &d.wine.prefix);
        seed(&mut self.wine.prefix_arch, &w.prefix_arch, &d.wine.prefix_arch);
        seed(&mut self.wine.esync, &w.esync, &d.wine.esync);
        seed(&mut self.wine.fsync, &w.fsync, &d.wine.fsync);
        seed(&mut self.wine.ntsync, &w.ntsync, &d.wine.ntsync);
        seed(&mut self.wine.dxvk, &w.dxvk, &d.wine.dxvk);
        seed(&mut self.wine.dxvk_version, &w.dxvk_version, &d.wine.dxvk_version);
        seed(&mut self.wine.vkd3d, &w.vkd3d, &d.wine.vkd3d);
        seed(&mut self.wine.vkd3d_version, &w.vkd3d_version, &d.wine.vkd3d_version);
        seed(&mut self.wine.d3d_extras, &w.d3d_extras, &d.wine.d3d_extras);
        seed(&mut self.wine.d3d_extras_version, &w.d3d_extras_version, &d.wine.d3d_extras_version);
        seed(&mut self.wine.dxvk_nvapi, &w.dxvk_nvapi, &d.wine.dxvk_nvapi);
        seed(&mut self.wine.dxvk_nvapi_version, &w.dxvk_nvapi_version, &d.wine.dxvk_nvapi_version);
        seed(&mut self.wine.fsr, &w.fsr, &d.wine.fsr);
        seed(&mut self.wine.battleye, &w.battleye, &d.wine.battleye);
        seed(&mut self.wine.easyanticheat, &w.easyanticheat, &d.wine.easyanticheat);
        seed(&mut self.wine.dpi_scaling, &w.dpi_scaling, &d.wine.dpi_scaling);
        seed(&mut self.wine.dpi, &w.dpi, &d.wine.dpi);
        seed(&mut self.wine.audio_driver, &w.audio_driver, &d.wine.audio_driver);
        seed(&mut self.wine.graphics_driver, &w.graphics_driver, &d.wine.graphics_driver);
        for (k, v) in &d.wine.dll_overrides {
            self.wine.dll_overrides.entry(k.clone()).or_insert_with(|| v.clone());
        }

        seed(&mut self.launch.command_prefix, &l.command_prefix, &d.launch.command_prefix);
        for (k, v) in &d.launch.env {
            self.launch.env.entry(k.clone()).or_insert_with(|| v.clone());
        }

        seed(&mut self.graphics.mangohud, &g.mangohud, &d.graphics.mangohud);
        seed(&mut self.graphics.gpu, &g.gpu, &d.graphics.gpu);

        let gs = &mut self.graphics.gamescope;
        let dgs = &d.graphics.gamescope;
        seed(&mut gs.enabled, &gs_def.enabled, &dgs.enabled);
        seed(&mut gs.width, &gs_def.width, &dgs.width);
        seed(&mut gs.height, &gs_def.height, &dgs.height);
        seed(&mut gs.game_width, &gs_def.game_width, &dgs.game_width);
        seed(&mut gs.game_height, &gs_def.game_height, &dgs.game_height);
        seed(&mut gs.fps, &gs_def.fps, &dgs.fps);
        seed(&mut gs.refresh_rate, &gs_def.refresh_rate, &dgs.refresh_rate);
        seed(&mut gs.fullscreen, &gs_def.fullscreen, &dgs.fullscreen);
        seed(&mut gs.borderless, &gs_def.borderless, &dgs.borderless);
        seed(&mut gs.integer_scaling, &gs_def.integer_scaling, &dgs.integer_scaling);
        seed(&mut gs.hdr, &gs_def.hdr, &dgs.hdr);
        seed(&mut gs.filter, &gs_def.filter, &dgs.filter);
        seed(&mut gs.fsr_sharpness, &gs_def.fsr_sharpness, &dgs.fsr_sharpness);

        seed(&mut self.system.gamemode, &s.gamemode, &d.system.gamemode);
        seed(&mut self.system.prevent_sleep, &s.prevent_sleep, &d.system.prevent_sleep);
        seed(&mut self.system.pulse_latency, &s.pulse_latency, &d.system.pulse_latency);
        seed(&mut self.system.cpu_limit, &s.cpu_limit, &d.system.cpu_limit);
    }
}

