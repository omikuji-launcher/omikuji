// gryphline launcher REST client for endfield.
//
// distinct from hoyo's hyp-connect API:
//  - response body is flat (no {code, message, data} envelope on launcher endpoints)
//  - numeric bytes come back as strings (can exceed js Number.MAX_SAFE_INTEGER), hence custom deserializers below
//  - get_latest does NOT take rand_str; only the resources endpoint needs it,and its not actually random: it's a stable per-release token extracted from a prior get_latest response's pkg.file_path

use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize};

use crate::gacha::manifest::GachaManifest;

const PLATFORM: &str = "Windows";

#[derive(Debug, Clone)]
pub struct EditionConfig {
    pub api_base: String,
    pub game_appcode: String,
    pub launcher_appcode: String,
    pub channel: u32,
    pub sub_channel: u32,
}

impl EditionConfig {
    pub fn from_manifest(manifest: &GachaManifest, edition_id: &str) -> Result<Self> {
        let cfg = manifest
            .editions
            .iter()
            .find(|e| e.id == edition_id)
            .map(|e| &e.strategy_config)
            .ok_or_else(|| anyhow!("edition {} not in manifest {}", edition_id, manifest.id))?;
        let s = |k: &str| -> Result<String> {
            cfg.get(k)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("missing strategy_config.{} in manifest {}", k, manifest.id))
        };
        let n = |k: &str| -> Result<u32> {
            cfg.get(k)
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
                .ok_or_else(|| anyhow!("missing strategy_config.{} in manifest {}", k, manifest.id))
        };
        Ok(Self {
            api_base: s("api_base")?,
            game_appcode: s("game_appcode")?,
            launcher_appcode: s("launcher_appcode")?,
            channel: n("channel")?,
            sub_channel: n("sub_channel")?,
        })
    }
}

fn str_to_u64_opt<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum V {
        S(String),
        N(u64),
    }
    match Option::<V>::deserialize(d)? {
        Some(V::S(s)) if !s.is_empty() => s.parse::<u64>().map_err(serde::de::Error::custom),
        Some(V::N(n)) => Ok(n),
        _ => Ok(0),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetLatestData {
    #[serde(default)]
    pub action: i32,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub request_version: String,
    #[serde(default)]
    pub pkg: Option<PkgInfo>,
    #[serde(default)]
    pub patch: Option<PatchInfo>,
    #[serde(default)]
    pub state: i32,
    #[serde(default)]
    pub launcher_action: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PkgInfo {
    #[serde(default, deserialize_with = "str_to_u64_opt")]
    pub total_size: u64,
    #[serde(default)]
    pub packs: Vec<PackFile>,
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub game_files_md5: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackFile {
    pub url: String,
    #[serde(default)]
    pub md5: String,
    #[serde(default, deserialize_with = "str_to_u64_opt")]
    pub package_size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PatchInfo {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub md5: String,
    #[serde(default, deserialize_with = "str_to_u64_opt")]
    pub package_size: u64,
    #[serde(default, deserialize_with = "str_to_u64_opt")]
    pub total_size: u64,
    #[serde(default)]
    pub patches: Vec<PackFile>,
    #[serde(default)]
    pub v2_patch_info_url: String,
    #[serde(default, deserialize_with = "str_to_u64_opt")]
    pub v2_patch_info_size: u64,
    #[serde(default)]
    pub v2_patch_info_md5: String,
}

fn encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            c if c.is_ascii_alphanumeric() || "-_.:~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}

fn build_get_latest_url(cfg: &EditionConfig, version: &str) -> String {
    // channel/sub_channel are integers here, strings in batch_proxy; not our concern
    let mut q = format!(
        "?appcode={}&launcher_appcode={}&channel={}&sub_channel={}&launcher_sub_channel={}",
        encode(&cfg.game_appcode),
        encode(&cfg.launcher_appcode),
        cfg.channel,
        cfg.sub_channel,
        cfg.sub_channel,
    );
    if !version.is_empty() {
        q.push_str("&version=");
        q.push_str(&encode(version));
    }
    format!("{}/game/get_latest{}", cfg.api_base, q)
}

pub async fn fetch_latest(cfg: &EditionConfig, installed_version: &str) -> Result<GetLatestData> {
    let url = build_get_latest_url(cfg, installed_version);
    let body = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| anyhow!("GET {} failed: {}", url, e))?
        .error_for_status()
        .map_err(|e| anyhow!("GET {} http error: {}", url, e))?
        .text()
        .await
        .map_err(|e| anyhow!("GET {} read failed: {}", url, e))?;

    serde_json::from_str::<GetLatestData>(&body)
        .map_err(|e| anyhow!("get_latest bad json: {} — body head: {}", e, head(&body)))
}

fn head(s: &str) -> String {
    s.chars().take(200).collect()
}

pub fn packs_from(resp: &GetLatestData) -> &[PackFile] {
    resp.pkg.as_ref().map(|p| &p.packs[..]).unwrap_or(&[])
}

pub fn patches_from(resp: &GetLatestData) -> &[PackFile] {
    resp.patch.as_ref().map(|p| &p.patches[..]).unwrap_or(&[])
}

// pkg.file_path is shaped .../{version}_{randstr}/files. pull the 16-char randstr out; it's stable per-release (not per-request) and get_latest_resources
// requires it as a query param. returns empty string if shape doesnt match.
pub fn rand_str_from(resp: &GetLatestData) -> String {
    let Some(pkg) = resp.pkg.as_ref() else {
        return String::new();
    };
    let fp = &pkg.file_path;
    // match _<rand>/<tail>$ via manual scan, no regex dep
    let Some(tail_slash) = fp.rfind('/') else {
        return String::new();
    };
    let before = &fp[..tail_slash];
    let Some(us) = before.rfind('_') else {
        return String::new();
    };
    before[us + 1..].to_string()
}

// per-resource CDN paths. each resource's path serves patch.json (per-file HDiffPatch descriptors). domain field is redundant with each path's host.
// patch_index_path is empty in practice, ignore it
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceList {
    #[serde(default)]
    pub resources: Vec<ResourceRef>,
    #[serde(default)]
    pub configs: String,
    #[serde(default)]
    pub res_version: String,
    #[serde(default)]
    pub patch_index_path: String,
    #[serde(default)]
    pub domain: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceRef {
    pub name: String,
    pub version: String,
    pub path: String,
}

fn build_get_latest_resources_url(
    cfg: &EditionConfig,
    game_version: &str,
    version: &str,
    rand_str: &str,
) -> String {
    format!(
        "{}/game/get_latest_resources?appcode={}&game_version={}&version={}&platform={}&rand_str={}",
        cfg.api_base,
        encode(&cfg.game_appcode),
        encode(game_version),
        encode(version),
        encode(PLATFORM),
        encode(rand_str),
    )
}

pub async fn fetch_resources(
    cfg: &EditionConfig,
    game_version: &str,
    version: &str,
    rand_str: &str,
) -> Result<ResourceList> {
    let url = build_get_latest_resources_url(cfg, game_version, version, rand_str);
    let body = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| anyhow!("GET {} failed: {}", url, e))?
        .error_for_status()
        .map_err(|e| anyhow!("GET {} http error: {}", url, e))?
        .text()
        .await
        .map_err(|e| anyhow!("GET {} read failed: {}", url, e))?;
    serde_json::from_str::<ResourceList>(&body).map_err(|e| {
        anyhow!(
            "get_latest_resources bad json: {} — body head: {}",
            e,
            head(&body)
        )
    })
}

// shape from live capture (resource=main, v1.2.4):
//   { "version": "6668018-7", "files": [ResourcePatchFile, ...] }
//
// each file carries HDiffPatch variants keyed implicitly by which base build the patch was generated against
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcePatchManifest {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub files: Vec<ResourcePatchFile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcePatchFile {
    pub name: String,
    pub md5: String,
    #[serde(default)]
    pub size: u64,
    // diffType=1 is chk (hdiff patch). other vcalues arent observed in the wild, Treat them as "skip, unknown".
    #[serde(default, rename = "diffType")]
    pub diff_type: u32,
    #[serde(default)]
    pub local_path: String,
    #[serde(default)]
    pub patch: Vec<ResourcePatchVariant>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcePatchVariant {
    pub base_file: String,
    pub base_md5: String,
    #[serde(default)]
    pub base_size: u64,
    #[serde(rename = "patch")]
    pub patch_path: String,
    #[serde(default)]
    pub patch_size: u64,
}

pub async fn fetch_resource_patch(resource_path: &str) -> Result<ResourcePatchManifest> {
    let url = format!("{}/patch.json", resource_path.trim_end_matches('/'));
    let body = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| anyhow!("GET {} failed: {}", url, e))?
        .error_for_status()
        .map_err(|e| anyhow!("GET {} http error: {}", url, e))?
        .text()
        .await
        .map_err(|e| anyhow!("GET {} read failed: {}", url, e))?;
    serde_json::from_str::<ResourcePatchManifest>(&body)
        .map_err(|e| anyhow!("patch.json bad json: {} — body head: {}", e, head(&body)))
}

#[derive(Debug, Clone, Copy)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

pub async fn fetch_install_size(manifest: &GachaManifest, edition_id: &str) -> Result<InstallSize> {
    let cfg = EditionConfig::from_manifest(manifest, edition_id)?;
    let resp = fetch_latest(&cfg, "").await?;
    let pkg = resp
        .pkg
        .ok_or_else(|| anyhow!("get_latest returned no pkg"))?;
    let download: u64 = pkg.packs.iter().map(|p| p.package_size).sum();
    Ok(InstallSize {
        download_bytes: download,
        install_bytes: pkg.total_size.saturating_sub(download),
    })
}
