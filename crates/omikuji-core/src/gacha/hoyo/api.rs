use anyhow::{Result, anyhow};

use super::HoyoEdition;

#[derive(Debug, Clone, Copy)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

pub async fn fetch_install_size(
    biz_id: &str,
    edition: HoyoEdition,
    voices: &[String],
) -> Result<InstallSize> {
    let branches = super::sophon::api::fetch_game_branches(edition).await?;
    let branch = branches
        .find_for(biz_id)
        .ok_or_else(|| anyhow!("game branch not found for biz_id {}", biz_id))?;
    let main = branch
        .main
        .as_ref()
        .ok_or_else(|| anyhow!("no main package info"))?;
    let build = super::sophon::api::fetch_build(edition, main).await?;

    let mut download = 0u64;
    let mut peak = 0u64;

    let mut accumulate = |entry: &super::sophon::api::SophonManifestEntry| {
        if let Some(s) = &entry.stats {
            download += s.compressed_size.parse::<u64>().unwrap_or(0);
            peak += s.uncompressed_size.parse::<u64>().unwrap_or(0);
        }
    };

    if let Some(game) = build.get_for("game") {
        accumulate(game);
    }
    for voice in voices {
        if let Some(audio) = build.get_for(voice) {
            accumulate(audio);
        }
    }

    Ok(InstallSize {
        download_bytes: download,
        install_bytes: peak.max(download),
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
