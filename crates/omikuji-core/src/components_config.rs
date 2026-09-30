use crate::archive_source::{ReleaseInfo, normalize_releases_url};
use crate::fs_util::write_atomic;
use crate::{dll_packs, runners};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchiveSource {
    pub name: String,
    pub kind: String,
    pub api_url: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub asset_priority: Vec<String>,
    #[serde(default)]
    pub require_asset_match: bool,
    #[serde(default)]
    pub prefix_install_version: String,
}

impl ArchiveSource {
    fn normalized(self) -> Self {
        Self {
            api_url: normalize_releases_url(&self.api_url),
            ..self
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ComponentsConfig {
    pub runners: Vec<ArchiveSource>,
    pub layers: Vec<ArchiveSource>,
}

impl Default for ComponentsConfig {
    fn default() -> Self {
        Self {
            runners: default_runners(),
            layers: default_layers(),
        }
    }
}

fn src(name: &str, kind: &str, api_url: &str) -> ArchiveSource {
    ArchiveSource {
        name: name.into(),
        kind: kind.into(),
        api_url: api_url.into(),
        desc: String::new(),
        asset_priority: Vec::new(),
        require_asset_match: false,
        prefix_install_version: String::new(),
    }
}

pub fn default_runners() -> Vec<ArchiveSource> {
    vec![
        src(
            "Proton-Spritz",
            "proton",
            "https://api.github.com/repos/NelloKudo/proton-cachyos/releases",
        ),
        src(
            "Proton-GE",
            "proton",
            "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases",
        ),
        src(
            "Dawn Winery Proton",
            "proton",
            "https://dawn.wine/api/v1/repos/dawn-winery/dwproton/releases",
        ),
        src(
            "Proton-Cachyos",
            "proton",
            "https://api.github.com/repos/CachyOS/proton-cachyos/releases",
        ),
        src(
            "Wine-Spritz",
            "wine",
            "https://api.github.com/repos/NelloKudo/spritz-wine/releases",
        ),
    ]
}

pub fn default_layers() -> Vec<ArchiveSource> {
    vec![
        src(
            "DXVK",
            "dxvk",
            "https://api.github.com/repos/doitsujin/dxvk/releases",
        ),
        src(
            "VKD3D-Proton",
            "vkd3d",
            "https://api.github.com/repos/HansKristian-Work/vkd3d-proton/releases",
        ),
        src(
            "DXVK-NVAPI",
            "dxvk_nvapi",
            "https://api.github.com/repos/jp7677/dxvk-nvapi/releases",
        ),
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
    match fs_err::read_to_string(&path) {
        Ok(body) => toml::from_str::<ComponentsConfig>(&body).unwrap_or_else(|e| {
            tracing::warn!("couldn't parse {}: {} - using defaults", path.display(), e);
            ComponentsConfig::default()
        }),
        Err(e) => {
            tracing::warn!("{e} - using defaults");
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
    write_atomic(&config_path(), body)?;
    *guard = Some(config);
    Ok(out)
}

pub fn save(config: &ComponentsConfig) -> anyhow::Result<()> {
    mutate(|c| {
        *c = config.clone();
        Ok(())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceCategory {
    Runners,
    Layers,
}

impl SourceCategory {
    pub const ALL: [Self; 2] = [Self::Runners, Self::Layers];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runners => "runners",
            Self::Layers => "layers",
        }
    }

    pub fn sources(self) -> Vec<ArchiveSource> {
        match self {
            Self::Runners => runners::list_sources(),
            Self::Layers => dll_packs::list_sources(),
        }
    }

    pub fn find(self, name: &str) -> Option<ArchiveSource> {
        self.sources().into_iter().find(|s| s.name == name)
    }

    pub fn source_root(self, source: &ArchiveSource) -> PathBuf {
        match self {
            Self::Runners => runners::source_root(source),
            Self::Layers => dll_packs::source_root(source),
        }
    }

    pub fn list_installed(self, source: &ArchiveSource) -> Vec<String> {
        match self {
            Self::Runners => runners::list_installed(source),
            Self::Layers => dll_packs::list_installed(source),
        }
    }

    pub async fn install_version(
        self,
        source: &ArchiveSource,
        release: &ReleaseInfo,
    ) -> anyhow::Result<PathBuf> {
        match self {
            Self::Runners => runners::install_version(source, release).await,
            Self::Layers => dll_packs::install_version(source, release).await,
        }
    }

    pub fn delete_version(self, source: &ArchiveSource, tag: &str) -> anyhow::Result<()> {
        match self {
            Self::Runners => runners::delete_version(source, tag),
            Self::Layers => dll_packs::delete_version(source, tag),
        }
    }
}

impl FromStr for SourceCategory {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> anyhow::Result<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == name)
            .ok_or_else(|| anyhow::anyhow!("unknown source category: {}", name))
    }
}

fn list_mut(config: &mut ComponentsConfig, category: SourceCategory) -> &mut Vec<ArchiveSource> {
    match category {
        SourceCategory::Runners => &mut config.runners,
        SourceCategory::Layers => &mut config.layers,
    }
}

pub fn add_source(category: SourceCategory, source: ArchiveSource) -> anyhow::Result<()> {
    let source = source.normalized();
    mutate(|config| {
        let list = list_mut(config, category);
        if source.name.trim().is_empty() {
            anyhow::bail!("source name can't be empty");
        }
        if source.api_url.trim().is_empty() {
            anyhow::bail!("source url can't be empty");
        }
        if list
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(&source.name))
        {
            anyhow::bail!("a source named \"{}\" already exists", source.name);
        }
        list.push(source);
        Ok(())
    })
}

pub fn update_source(
    category: SourceCategory,
    name: &str,
    source: ArchiveSource,
) -> anyhow::Result<()> {
    let source = source.normalized();
    mutate(|config| {
        let list = list_mut(config, category);
        if source.api_url.trim().is_empty() {
            anyhow::bail!("source url can't be empty");
        }
        let existing = list
            .iter_mut()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow::anyhow!("no source named \"{}\"", name))?;
        *existing = ArchiveSource {
            name: existing.name.clone(),
            prefix_install_version: existing.prefix_install_version.clone(),
            ..source
        };
        Ok(())
    })
}

pub fn remove_source(category: SourceCategory, name: &str) -> anyhow::Result<()> {
    mutate(|config| {
        let list = list_mut(config, category);
        let before = list.len();
        list.retain(|s| s.name != name);
        if list.len() == before {
            anyhow::bail!("no source named \"{}\"", name);
        }
        Ok(())
    })
}

pub fn set_prefix_install_version(source_name: &str, tag: &str) -> anyhow::Result<()> {
    mutate(|config| {
        if let Some(source) = config.layers.iter_mut().find(|s| s.name == source_name) {
            source.prefix_install_version = tag.to_string();
        }
        Ok(())
    })
}
