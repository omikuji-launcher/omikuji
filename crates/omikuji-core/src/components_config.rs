use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchiveSource {
    pub name: String,
    pub kind: String,
    pub api_url: String,
    pub asset_pattern: String,
    pub extract: String,
    #[serde(default)]
    pub desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ComponentsConfig {
    pub runners: Vec<ArchiveSource>,
    pub layers: Vec<ArchiveSource>,
    pub active: HashMap<String, String>,
}

impl Default for ComponentsConfig {
    fn default() -> Self {
        Self {
            runners: default_runners(),
            layers: default_layers(),
            active: HashMap::new(),
        }
    }
}

fn src(name: &str, kind: &str, api_url: &str, asset_pattern: &str, extract: &str) -> ArchiveSource {
    ArchiveSource {
        name: name.into(),
        kind: kind.into(),
        api_url: api_url.into(),
        asset_pattern: asset_pattern.into(),
        extract: extract.into(),
        desc: String::new(),
    }
}

pub fn default_runners() -> Vec<ArchiveSource> {
    vec![
        src("Proton-Spritz", "proton", "https://api.github.com/repos/NelloKudo/proton-cachyos/releases", "-x86_64.tar.xz", "tar_xz"),
        src("Proton-GE", "proton", "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases", ".tar.gz", "tar_gz"),
        src("Dawn Winery Proton", "proton", "https://dawn.wine/api/v1/repos/dawn-winery/dwproton/releases", ".tar.xz", "tar_xz"),
        src("Proton-Cachyos", "proton", "https://api.github.com/repos/CachyOS/proton-cachyos/releases", ".tar.xz", "tar_xz"),
    ]
}

pub fn default_layers() -> Vec<ArchiveSource> {
    vec![
        src("DXVK", "dxvk", "https://api.github.com/repos/doitsujin/dxvk/releases", ".tar.gz", "tar_gz"),
        src("VKD3D-Proton", "vkd3d", "https://api.github.com/repos/HansKristian-Work/vkd3d-proton/releases", ".tar.zst", "tar_zst"),
        src("DXVK-NVAPI", "dxvk_nvapi", "https://api.github.com/repos/jp7677/dxvk-nvapi/releases", ".tar.gz", "tar_gz"),
    ]
}

pub fn config_path() -> PathBuf {
    crate::data_dir().join("components.toml")
}

static CACHE: Mutex<Option<ComponentsConfig>> = Mutex::new(None);

pub fn get() -> ComponentsConfig {
    let mut guard = CACHE.lock().unwrap();
    if let Some(c) = guard.as_ref() {
        return c.clone();
    }
    let c = load_from_disk();
    *guard = Some(c.clone());
    c
}

pub fn reload() {
    *CACHE.lock().unwrap() = None;
}

fn load_from_disk() -> ComponentsConfig {
    let path = config_path();
    if !path.exists() {
        return ComponentsConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(body) => toml::from_str::<ComponentsConfig>(&body).unwrap_or_else(|e| {
            tracing::warn!("couldn't parse {}: {} - using defaults", path.display(), e);
            ComponentsConfig::default()
        }),
        Err(e) => {
            tracing::warn!("couldn't read {}: {} - using defaults", path.display(), e);
            ComponentsConfig::default()
        }
    }
}

fn mutate<T>(f: impl FnOnce(&mut ComponentsConfig) -> anyhow::Result<T>) -> anyhow::Result<T> {
    let mut guard = CACHE.lock().unwrap();
    let mut config = match guard.as_ref() {
        Some(c) => c.clone(),
        None => load_from_disk(),
    };
    let out = f(&mut config)?;
    let body = toml::to_string_pretty(&config).map_err(std::io::Error::other)?;
    crate::fs_util::write_atomic(&config_path(), body)?;
    *guard = Some(config);
    Ok(out)
}

pub fn save(config: &ComponentsConfig) -> anyhow::Result<()> {
    mutate(|c| {
        *c = config.clone();
        Ok(())
    })
}

fn list_mut<'a>(
    config: &'a mut ComponentsConfig,
    category: &str,
) -> Option<&'a mut Vec<ArchiveSource>> {
    match category {
        "runners" => Some(&mut config.runners),
        "layers" => Some(&mut config.layers),
        _ => None,
    }
}

pub fn add_source(category: &str, source: ArchiveSource) -> anyhow::Result<()> {
    mutate(|config| {
        let list = list_mut(config, category)
            .ok_or_else(|| anyhow::anyhow!("unknown source category: {}", category))?;
        if source.name.trim().is_empty() {
            anyhow::bail!("source name can't be empty");
        }
        if source.api_url.trim().is_empty() {
            anyhow::bail!("source url can't be empty");
        }
        if list.iter().any(|s| s.name.eq_ignore_ascii_case(&source.name)) {
            anyhow::bail!("a source named \"{}\" already exists", source.name);
        }
        list.push(source);
        Ok(())
    })
}

pub fn remove_source(category: &str, name: &str) -> anyhow::Result<()> {
    mutate(|config| {
        let list = list_mut(config, category)
            .ok_or_else(|| anyhow::anyhow!("unknown source category: {}", category))?;
        let before = list.len();
        list.retain(|s| s.name != name);
        if list.len() == before {
            anyhow::bail!("no source named \"{}\"", name);
        }
        Ok(())
    })
}

pub fn active_version(source_name: &str) -> String {
    get().active.get(source_name).cloned().unwrap_or_default()
}

pub fn set_active_version(source_name: &str, tag: &str) -> anyhow::Result<()> {
    mutate(|config| {
        if tag.is_empty() {
            config.active.remove(source_name);
        } else {
            config.active.insert(source_name.to_string(), tag.to_string());
        }
        Ok(())
    })
}
