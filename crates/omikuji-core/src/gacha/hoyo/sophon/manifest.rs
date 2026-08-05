use anyhow::{Result, anyhow};
use prost::Message;

use super::api::{SophonDiff, SophonManifestEntry};
use super::protos::SophonManifest as SophonManifestProto;
use super::protos::SophonPatchProto;

pub async fn fetch_patch_manifest(diff: &SophonDiff) -> Result<SophonPatchProto> {
    let url = diff.manifest_download.url_for(&diff.manifest.id);
    let compressed = diff.manifest_download.compression == 1;

    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("fetch manifest {} failed: {}", url, e))?
        .error_for_status()
        .map_err(|e| anyhow!("fetch manifest {} http error: {}", url, e))?
        .bytes()
        .await
        .map_err(|e| anyhow!("fetch manifest {} read failed: {}", url, e))?;

    let payload = if compressed {
        zstd::decode_all(&*bytes).map_err(|e| anyhow!("manifest zstd decode failed: {}", e))?
    } else {
        bytes.to_vec()
    };

    SophonPatchProto::decode(&*payload)
        .map_err(|e| anyhow!("manifest protobuf decode failed: {}", e))
}

pub async fn fetch_build_manifest(entry: &SophonManifestEntry) -> Result<SophonManifestProto> {
    let url = entry.manifest_download.url_for(&entry.manifest.id);
    let compressed = entry.manifest_download.compression == 1;

    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("fetch manifest {} failed: {}", url, e))?
        .error_for_status()
        .map_err(|e| anyhow!("fetch manifest {} http error: {}", url, e))?
        .bytes()
        .await
        .map_err(|e| anyhow!("fetch manifest {} read failed: {}", url, e))?;

    let payload = if compressed {
        zstd::decode_all(&*bytes).map_err(|e| anyhow!("manifest zstd decode failed: {}", e))?
    } else {
        bytes.to_vec()
    };

    SophonManifestProto::decode(&*payload)
        .map_err(|e| anyhow!("manifest protobuf decode failed: {}", e))
}
