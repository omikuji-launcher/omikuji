use std::collections::HashMap;
use std::iter;

use anyhow::{Result, anyhow};

use super::HoyoEdition;
use super::sophon::{
    self,
    api::{SophonBuild, SophonManifestEntry},
};
use crate::gacha::manifest::ManifestVoice;

#[derive(Debug, Clone, Copy, Default)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

pub async fn fetch_install_size(
    biz_id: &str,
    edition: HoyoEdition,
    voices: &[String],
) -> Result<InstallSize> {
    let build = fetch_live_build(biz_id, edition).await?;
    let total = iter::once("game")
        .chain(voices.iter().map(String::as_str))
        .filter_map(|field| build.get_for(field))
        .map(entry_size)
        .fold(InstallSize::default(), |a, b| InstallSize {
            download_bytes: a.download_bytes + b.download_bytes,
            install_bytes: a.install_bytes + b.install_bytes,
        });
    Ok(InstallSize {
        install_bytes: total.install_bytes.max(total.download_bytes),
        ..total
    })
}

pub async fn voice_sizes(
    biz_id: &str,
    edition: HoyoEdition,
    voices: &[ManifestVoice],
) -> Result<HashMap<String, u64>> {
    let build = fetch_live_build(biz_id, edition).await?;
    Ok(voices
        .iter()
        .filter_map(|v| {
            build
                .get_for(&v.id)
                .map(|e| (v.id.clone(), entry_size(e).download_bytes))
        })
        .collect())
}

async fn fetch_live_build(biz_id: &str, edition: HoyoEdition) -> Result<SophonBuild> {
    let branches = sophon::api::fetch_game_branches(edition).await?;
    let branch = branches
        .find_for(biz_id)
        .ok_or_else(|| anyhow!("game branch not found for biz_id {}", biz_id))?;
    let main = branch
        .main
        .as_ref()
        .ok_or_else(|| anyhow!("no main package info"))?;
    sophon::api::fetch_build(edition, main).await
}

fn entry_size(entry: &SophonManifestEntry) -> InstallSize {
    entry
        .stats
        .as_ref()
        .map_or(InstallSize::default(), |s| InstallSize {
            download_bytes: s.compressed_size.parse().unwrap_or(0),
            install_bytes: s.uncompressed_size.parse().unwrap_or(0),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    // genshin global biz_id, hardcoded for live test only
    const GENSHIN_GLOBAL: &str = "gopR6Cufr3";

    #[tokio::test]
    #[ignore]
    async fn fetch_genshin_global_size_live() {
        let r =
            fetch_install_size(GENSHIN_GLOBAL, HoyoEdition::Global, &["en-us".to_string()]).await;
        match r {
            Ok(s) => {
                println!("download={} install={}", s.download_bytes, s.install_bytes);
                assert!(s.install_bytes > 0, "install size should be non-zero");
            }
            Err(e) => panic!("fetch failed: {}", e),
        }
    }
}
