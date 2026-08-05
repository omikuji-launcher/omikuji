use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::super::HoyoEdition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBranches {
    pub game_branches: Vec<GameBranchInfo>,
}

impl GameBranches {
    pub fn find_for(&self, biz_id: &str) -> Option<&GameBranchInfo> {
        self.game_branches.iter().find(|b| b.game.id == biz_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBranchInfo {
    pub game: GameRef,
    pub main: Option<PackageInfo>,
    pub pre_download: Option<PackageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRef {
    pub id: String,
    #[serde(default)]
    pub biz: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub package_id: String,
    pub branch: String,
    pub password: String,
    pub tag: String,
    // if current version isnt in diff_tags, teh server cant produce a diff; full reinstall needed
    #[serde(default)]
    pub diff_tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<PackageCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCategory {
    pub category_id: String,
    pub matching_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SophonDiffs {
    #[serde(default)]
    pub build_id: String,
    #[serde(default)]
    pub patch_id: String,
    #[serde(default)]
    pub tag: String,
    pub manifests: Vec<SophonDiff>,
}

impl SophonDiffs {
    pub fn get_for(&self, matching_field: &str) -> Option<&SophonDiff> {
        self.manifests
            .iter()
            .find(|m| m.matching_field == matching_field)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SophonBuild {
    #[serde(default)]
    pub build_id: String,
    #[serde(default)]
    pub tag: String,
    pub manifests: Vec<SophonManifestEntry>,
}

impl SophonBuild {
    pub fn get_for(&self, matching_field: &str) -> Option<&SophonManifestEntry> {
        self.manifests
            .iter()
            .find(|m| m.matching_field == matching_field)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SophonManifestEntry {
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub category_name: String,
    #[serde(default)]
    pub matching_field: String,
    pub manifest: ManifestRef,
    pub chunk_download: DownloadInfo,
    pub manifest_download: DownloadInfo,
    #[serde(default)]
    pub stats: Option<ManifestStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SophonDiff {
    #[serde(default)]
    pub category_id: String,
    #[serde(default)]
    pub category_name: String,
    pub matching_field: String,
    pub manifest: ManifestRef,
    pub diff_download: DownloadInfo,
    pub manifest_download: DownloadInfo,
    #[serde(default)]
    pub stats: BTreeMap<String, ManifestStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRef {
    pub id: String,
    #[serde(default)]
    pub checksum: String,
    pub compressed_size: String,
    pub uncompressed_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    #[serde(default)]
    pub encryption: u8,
    #[serde(default)]
    pub password: String,
    // 1 = zstd-compressed, 0 = raw
    #[serde(default)]
    pub compression: u8,
    pub url_prefix: String,
    pub url_suffix: String,
}

impl DownloadInfo {
    pub fn url_for(&self, id: &str) -> String {
        format!("{}{}/{}", self.url_prefix, self.url_suffix, id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestStats {
    pub compressed_size: String,
    pub uncompressed_size: String,
    #[serde(default)]
    pub file_count: String,
    #[serde(default)]
    pub chunk_count: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    retcode: i32,
    message: String,
    data: Option<T>,
}

async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let resp: ApiResponse<T> = reqwest::get(url)
        .await
        .map_err(|e| anyhow!("sophon GET {} failed: {}", url, e))?
        .json()
        .await
        .map_err(|e| anyhow!("sophon GET {} bad json: {}", url, e))?;
    if resp.retcode != 0 {
        return Err(anyhow!(
            "sophon api error {}: {}",
            resp.retcode,
            resp.message
        ));
    }
    resp.data
        .ok_or_else(|| anyhow!("sophon api returned no data"))
}

async fn post_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T> {
    let client = reqwest::Client::new();
    let resp: ApiResponse<T> = client
        .post(url)
        .send()
        .await
        .map_err(|e| anyhow!("sophon POST {} failed: {}", url, e))?
        .json()
        .await
        .map_err(|e| anyhow!("sophon POST {} bad json: {}", url, e))?;
    if resp.retcode != 0 {
        return Err(anyhow!(
            "sophon api error {}: {}",
            resp.retcode,
            resp.message
        ));
    }
    resp.data
        .ok_or_else(|| anyhow!("sophon api returned no data"))
}

pub async fn fetch_game_branches(edition: HoyoEdition) -> Result<GameBranches> {
    get_json(&super::game_branches_url(edition)).await
}

pub async fn fetch_patch_build(edition: HoyoEdition, pkg: &PackageInfo) -> Result<SophonDiffs> {
    post_json(&super::patch_build_url(edition, pkg)).await
}

pub async fn fetch_build(edition: HoyoEdition, pkg: &PackageInfo) -> Result<SophonBuild> {
    get_json(&super::build_url(edition, pkg)).await
}
