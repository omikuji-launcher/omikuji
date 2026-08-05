// kuro launcher api, two-stage resolve:
//   1. GET index.json: cdn + resource-index url + base-url for files
//   2. GET indexFile.json: resource[] listing every file to fetch
//
// the install source then walks resource[] and dowloads each entry to
// install_path/{resource.dest} directly. no archive extraction phase.

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::gacha::manifest::GachaManifest;

#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub version: String,
    pub cdn_url: String,
    pub index_file_url: String,
    pub base_url: String,
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub patch_configs: Vec<PatchConfig>,
}

#[derive(Debug, Clone)]
pub struct PatchConfig {
    pub version: String,
    // relative to cdn_url
    pub index_file_rel: String,
    pub base_url_rel: String,
    pub download_size: u64,
    pub un_compress_size: u64,
}

impl ResourceInfo {
    pub fn matching_patch(&self, from_version: &str) -> Option<&PatchConfig> {
        let target = crate::gacha::strategies::normalize_version(from_version);
        self.patch_configs
            .iter()
            .find(|p| crate::gacha::strategies::normalize_version(&p.version) == target)
    }
}

pub async fn fetch_resource_info(
    manifest: &GachaManifest,
    edition_id: &str,
) -> Result<ResourceInfo> {
    let url = super::index_url_from_manifest(manifest, edition_id)?;

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("fetch {}: {}", url, e))?;
    if !resp.status().is_success() {
        anyhow::bail!("fetch {}: http {}", url, resp.status());
    }
    let data: RawIndex = resp
        .json()
        .await
        .map_err(|e| anyhow!("parse index.json: {}", e))?;

    // kuro lists several cdns sorted by priority (P field, lowest wins). deterministic-
    // pick-lowest is simpler than weighted random and adeuqate for a self-hosted launcher
    let mut cdns: Vec<&RawCdnEntry> = data.default.cdn_list.iter().collect();
    cdns.sort_by_key(|c| c.priority);
    let cdn_url = cdns
        .first()
        .map(|c| c.url.clone())
        .ok_or_else(|| anyhow!("cdnList empty in index.json"))?;

    // two api shapes in the wild:
    //   nested: version/indexFile/baseUrl under default.config  (WuWa 3.x /launcher/game/)
    //   flat:   version/resources/resourcesBasePath under default (PGR + WuWa 2.x /pcstarter/...)
    // same install behavior either way; pick out the fields that exist and normalize
    let (version, index_file_rel, base_url_rel, dl_bytes, inst_bytes, patch_configs) = match data
        .default
        .config
        .as_ref()
        .and_then(|c| c.version.as_ref())
    {
        Some(_) => {
            let cfg = data.default.config.clone().unwrap();
            (
                cfg.version.unwrap_or_default(),
                cfg.index_file
                    .ok_or_else(|| anyhow!("nested config missing indexFile"))?,
                cfg.base_url
                    .ok_or_else(|| anyhow!("nested config missing baseUrl"))?,
                cfg.size,
                cfg.un_compress_size,
                cfg.patch_config
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| PatchConfig {
                        version: p.version,
                        index_file_rel: p.index_file,
                        base_url_rel: p.base_url,
                        download_size: p.size,
                        un_compress_size: p.un_compress_size,
                    })
                    .collect(),
            )
        }
        None => {
            // flat shape has no aggregate sizes; caller walks indexFile if needed
            let def = &data.default;
            let v = def
                .version
                .clone()
                .ok_or_else(|| anyhow!("index.json has no version (neither nested nor flat)"))?;
            let resources = def
                .resources
                .clone()
                .ok_or_else(|| anyhow!("flat shape missing `resources`"))?;
            let base = def
                .resources_base_path
                .clone()
                .ok_or_else(|| anyhow!("flat shape missing `resourcesBasePath`"))?;
            // resourcesBasePath omits the trailing slash the downloader needs
            let base_with_slash = if base.ends_with('/') {
                base
            } else {
                format!("{}/", base)
            };
            (v, resources, base_with_slash, 0u64, 0u64, Vec::new())
        }
    };

    let index_file_url = format!("{}{}", cdn_url, index_file_rel);
    let base_url = format!("{}{}", cdn_url, base_url_rel);

    Ok(ResourceInfo {
        version,
        cdn_url,
        index_file_url,
        base_url,
        download_bytes: dl_bytes,
        install_bytes: inst_bytes,
        patch_configs,
    })
}

#[derive(Deserialize)]
struct RawIndex {
    default: RawDefault,
}

#[derive(Deserialize)]
struct RawDefault {
    #[serde(rename = "cdnList")]
    cdn_list: Vec<RawCdnEntry>,
    #[serde(default)]
    config: Option<RawConfig>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    resources: Option<String>,
    #[serde(default, rename = "resourcesBasePath")]
    resources_base_path: Option<String>,
}

#[derive(Clone, Deserialize)]
struct RawConfig {
    #[serde(default)]
    version: Option<String>,
    #[serde(default, rename = "indexFile")]
    index_file: Option<String>,
    #[serde(default, rename = "baseUrl")]
    base_url: Option<String>,
    #[serde(default)]
    size: u64,
    #[serde(rename = "unCompressSize", default)]
    un_compress_size: u64,
    #[serde(rename = "patchConfig", default)]
    patch_config: Option<Vec<RawPatchConfig>>,
}

#[derive(Deserialize)]
struct RawCdnEntry {
    url: String,
    // lower is preferred, sometimes missing/0
    #[serde(rename = "P", default)]
    priority: i64,
}

#[derive(Clone, Deserialize)]
struct RawPatchConfig {
    version: String,
    #[serde(rename = "indexFile")]
    index_file: String,
    #[serde(rename = "baseUrl")]
    base_url: String,
    #[serde(default)]
    size: u64,
    #[serde(rename = "unCompressSize", default)]
    un_compress_size: u64,
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
    #[serde(deserialize_with = "deserialize_u64_or_string")]
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatchGroup {
    pub dest: String,
    #[serde(default, rename = "srcFiles")]
    pub src_files: Vec<ResourceFile>,
    #[serde(default, rename = "dstFiles")]
    pub dst_files: Vec<ResourceFile>,
}

// server returns size as either a number or a stringified number depending on the endpoint
fn deserialize_u64_or_string<'de, D>(d: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Number(n) => {
            n.as_u64().ok_or_else(|| D::Error::custom("size not a u64"))
        }
        serde_json::Value::String(s) => s.parse().map_err(D::Error::custom),
        other => Err(D::Error::custom(format!(
            "size unexpected type: {:?}",
            other
        ))),
    }
}

pub async fn fetch_index_file(index_file_url: &str) -> Result<IndexFile> {
    fetch_json(index_file_url).await
}

pub async fn fetch_patch_index(index_file_url: &str) -> Result<PatchIndexFile> {
    fetch_json(index_file_url).await
}

async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| anyhow!("fetch indexFile: {}", e))?;
    if !resp.status().is_success() {
        anyhow::bail!("fetch indexFile: http {}", resp.status());
    }
    // pull bytes and parse with serde_json directly so parse errorss carry line/column and
    // a peek of the body. kuro responses hit 5-50 MB and bad json would otherwise fail with an opaque "error decoding response"
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| anyhow!("read indexFile body: {}", e))?;
    serde_json::from_slice(&bytes).map_err(|e| {
        let head: String = String::from_utf8_lossy(&bytes[..bytes.len().min(300)]).into_owned();
        anyhow!("parse indexFile: {} — body head: {}", e, head)
    })
}

#[derive(Debug, Clone, Copy)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

pub async fn fetch_install_size(manifest: &GachaManifest, edition_id: &str) -> Result<InstallSize> {
    let info = fetch_resource_info(manifest, edition_id).await?;
    if info.download_bytes > 0 {
        return Ok(InstallSize {
            download_bytes: info.download_bytes,
            install_bytes: info.install_bytes.max(info.download_bytes),
        });
    }
    let index = fetch_index_file(&info.index_file_url).await?;
    let total: u64 = index.resource.iter().map(|r| r.size).sum();
    Ok(InstallSize {
        download_bytes: total,
        install_bytes: total,
    })
}
