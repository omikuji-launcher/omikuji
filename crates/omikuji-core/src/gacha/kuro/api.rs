use std::collections::HashMap;
use std::iter;

use anyhow::{Result, anyhow};
use indexmap::IndexMap;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::gacha::file_sync::deserialize_size;
use crate::gacha::manifest::GachaManifest;
use crate::gacha::strategies::normalize_version;
use crate::http;

const LEGACY_PACK: &str = "common";

#[derive(Debug, Clone)]
pub struct KuroIndex {
    pub packs: IndexMap<String, Pack>,
    pub bundles: IndexMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Pack {
    pub version: String,
    pub cdn_url: String,
    pub index_file_url: String,
    pub base_url: String,
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub patch_configs: Vec<PatchConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchConfig {
    pub version: String,
    #[serde(rename = "indexFile")]
    pub index_file_rel: String,
    #[serde(rename = "baseUrl")]
    pub base_url_rel: String,
    #[serde(rename = "size", default)]
    pub download_size: u64,
}

impl KuroIndex {
    pub fn base_id(&self) -> Result<&str> {
        let mut shared = self
            .packs
            .keys()
            .filter(|id| self.bundles.values().all(|b| b.contains(id)));
        match (shared.next(), shared.next()) {
            (Some(id), None) => Ok(id),
            _ => Err(anyhow!(
                "index.json has no single pack shared by every bundle"
            )),
        }
    }

    pub fn base(&self) -> Result<&Pack> {
        self.pack(self.base_id()?)
    }

    pub fn pack(&self, id: &str) -> Result<&Pack> {
        self.packs
            .get(id)
            .ok_or_else(|| anyhow!("index.json has no pack {id}"))
    }
}

impl Pack {
    pub fn matching_patch(&self, from_version: &str) -> Option<&PatchConfig> {
        let target = normalize_version(from_version);
        self.patch_configs
            .iter()
            .find(|p| normalize_version(&p.version) == target)
    }
}

pub async fn fetch_index(index_url: &str) -> Result<KuroIndex> {
    fetch_json::<RawIndex>(index_url).await?.resolve()
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawIndex {
    Packs {
        #[serde(rename = "cdnList")]
        cdn_list: Vec<RawCdnEntry>,
        #[serde(rename = "resourcePacks")]
        resource_packs: IndexMap<String, RawPack>,
        #[serde(default)]
        bundles: IndexMap<String, RawBundle>,
    },
    Legacy {
        default: RawDefault,
    },
}

#[derive(Deserialize)]
struct RawDefault {
    #[serde(rename = "cdnList")]
    cdn_list: Vec<RawCdnEntry>,
    config: RawPack,
}

#[derive(Deserialize)]
struct RawBundle {
    #[serde(rename = "resourcePacks")]
    resource_packs: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPack {
    version: String,
    index_file: String,
    base_url: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    un_compress_size: u64,
    #[serde(default)]
    patch_config: Vec<PatchConfig>,
}

#[derive(Deserialize)]
struct RawCdnEntry {
    url: String,
    #[serde(rename = "P", default)]
    priority: i64,
}

impl RawIndex {
    fn resolve(self) -> Result<KuroIndex> {
        let (cdn_list, packs, bundles) = match self {
            Self::Packs {
                cdn_list,
                resource_packs,
                bundles,
            } => (
                cdn_list,
                resource_packs,
                bundles
                    .into_iter()
                    .map(|(id, b)| (id, b.resource_packs))
                    .collect(),
            ),
            Self::Legacy { default } => (
                default.cdn_list,
                IndexMap::from([(LEGACY_PACK.to_string(), default.config)]),
                IndexMap::new(),
            ),
        };
        let cdn_url = cdn_list
            .into_iter()
            .min_by_key(|c| c.priority)
            .map(|c| c.url)
            .ok_or_else(|| anyhow!("cdnList empty in index.json"))?;
        Ok(KuroIndex {
            packs: packs
                .into_iter()
                .map(|(id, raw)| (id, raw.resolve(&cdn_url)))
                .collect(),
            bundles,
        })
    }
}

impl RawPack {
    fn resolve(self, cdn_url: &str) -> Pack {
        Pack {
            version: self.version,
            cdn_url: cdn_url.to_string(),
            index_file_url: format!("{cdn_url}{}", self.index_file),
            base_url: format!("{cdn_url}{}", self.base_url),
            download_bytes: self.size,
            install_bytes: self.un_compress_size,
            patch_configs: self.patch_config,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexFile {
    pub resource: Vec<ResourceFile>,
    #[serde(default, rename = "deleteFiles")]
    pub delete_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceFile {
    pub dest: String,
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u64,
    #[serde(default)]
    pub md5: String,
    #[serde(default, rename = "fromFolder")]
    pub from_folder: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchIndexFile {
    pub resource: Vec<ResourceFile>,
    #[serde(default, rename = "deleteFiles")]
    pub delete_files: Vec<String>,
    #[serde(default, rename = "groupInfos")]
    pub group_infos: Vec<PatchGroup>,
    #[serde(default, rename = "zipInfos")]
    pub zip_infos: Vec<serde_json::Value>,
    #[serde(default, rename = "patchInfos")]
    pub patch_infos: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchGroup {
    pub dest: String,
    #[serde(default, rename = "srcFiles")]
    pub src_files: Vec<ResourceFile>,
    #[serde(default, rename = "dstFiles")]
    pub dst_files: Vec<ResourceFile>,
}

pub async fn fetch_index_file(index_file_url: &str) -> Result<IndexFile> {
    fetch_json(index_file_url).await
}

pub async fn fetch_patch_index(index_file_url: &str) -> Result<PatchIndexFile> {
    fetch_json(index_file_url).await
}

async fn fetch_json<T: DeserializeOwned>(url: &str) -> Result<T> {
    let resp = http::client()
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow!("fetch {url}: {e}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("fetch {url}: http {}", resp.status());
    }
    // parse by hand so errors carry line/column; these bodies hit 50 MB and reqwest's error is opaque
    let bytes = resp.bytes().await.map_err(|e| anyhow!("read {url}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| {
        let head: String = String::from_utf8_lossy(&bytes[..bytes.len().min(300)]).into_owned();
        anyhow!("parse {url}: {e} - body head: {head}")
    })
}

#[derive(Debug, Clone, Copy)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

pub async fn fetch_install_size(
    manifest: &GachaManifest,
    edition_id: &str,
    picked: &[String],
) -> Result<InstallSize> {
    let config = super::KuroConfig::load(manifest, edition_id)?;
    let index = fetch_index(&config.index_url).await?;
    let tiers = config.install_packs(picked, &manifest.display_name)?;
    let mut total = InstallSize {
        download_bytes: 0,
        install_bytes: 0,
    };
    for id in iter::once(index.base_id()?).chain(tiers) {
        let size = pack_size(index.pack(id)?).await?;
        total.download_bytes += size.download_bytes;
        total.install_bytes += size.install_bytes;
    }
    Ok(total)
}

pub async fn pack_sizes(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Result<HashMap<String, u64>> {
    let config = super::KuroConfig::load(manifest, edition_id)?;
    let index = fetch_index(&config.index_url).await?;
    let mut sizes = HashMap::new();
    for id in config.packs.keys() {
        sizes.insert(id.clone(), pack_size(index.pack(id)?).await?.download_bytes);
    }
    Ok(sizes)
}

async fn pack_size(pack: &Pack) -> Result<InstallSize> {
    if pack.download_bytes > 0 {
        return Ok(InstallSize {
            download_bytes: pack.download_bytes,
            install_bytes: pack.install_bytes.max(pack.download_bytes),
        });
    }
    let files = fetch_index_file(&pack.index_file_url).await?;
    let total: u64 = files.resource.iter().map(|r| r.size).sum();
    Ok(InstallSize {
        download_bytes: total,
        install_bytes: total,
    })
}
