use anyhow::{Result, anyhow};
use serde::Deserialize;

use super::{HoyoEdition, VoiceLocale};

#[derive(Debug, Clone, Copy)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

impl InstallSize {
    pub fn peak_bytes(&self) -> u64 {
        self.download_bytes + self.install_bytes
    }
}

pub async fn fetch_install_size(
    biz_id: &str,
    edition: HoyoEdition,
    voices: &[VoiceLocale],
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
    for locale in voices {
        if let Some(audio) = build.get_for(locale.api_name()) {
            accumulate(audio);
        }
    }

    Ok(InstallSize {
        download_bytes: download,
        install_bytes: peak.max(download),
    })
}

#[derive(Debug, Clone)]
pub struct GamePackageInfo {
    pub version: String,
    pub game_packages: Vec<PackageFile>,
    pub audio_packages: Vec<AudioPackage>,
}

#[derive(Debug, Clone)]
pub struct PackageFile {
    pub url: String,
    pub md5: String,
    pub size: u64,
    pub decompressed_size: u64,
}

#[derive(Debug, Clone)]
pub struct AudioPackage {
    pub locale: VoiceLocale,
    pub file: PackageFile,
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub exe_name: String,
    pub audio_pkg_path: String,
}

pub async fn fetch_packages(biz_id: &str, edition: HoyoEdition) -> Result<GamePackageInfo> {
    let url = format!(
        "{}/getGamePackages?launcher_id={}",
        edition.api_base(),
        edition.launcher_id()
    );

    let resp: ApiResponse<GamePackagesData> = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("failed to reach hoyo api: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow!("failed to parse hoyo api response: {}", e))?;

    if resp.retcode != 0 {
        return Err(anyhow!("hoyo api error {}: {}", resp.retcode, resp.message));
    }

    let data = resp
        .data
        .ok_or_else(|| anyhow!("hoyo api returned no data"))?;

    let entry = data
        .game_packages
        .into_iter()
        .find(|gp| gp.game.id == biz_id)
        .ok_or_else(|| anyhow!("game {} not found in api response", biz_id))?;

    let major = entry
        .main
        .major
        .ok_or_else(|| anyhow!("no major version in api response"))?;

    let game_packages = major
        .game_pkgs
        .into_iter()
        .filter(|p| !p.url.is_empty())
        .map(|p| PackageFile {
            url: p.url,
            md5: p.md5,
            size: parse_size(&p.size),
            decompressed_size: parse_size(&p.decompressed_size),
        })
        .collect();

    let audio_packages = major
        .audio_pkgs
        .into_iter()
        .filter(|p| !p.url.is_empty())
        .filter_map(|p| {
            let locale = VoiceLocale::all()
                .iter()
                .find(|vl| vl.api_name() == p.language)?;
            Some(AudioPackage {
                locale: *locale,
                file: PackageFile {
                    url: p.url,
                    md5: p.md5,
                    size: parse_size(&p.size),
                    decompressed_size: parse_size(&p.decompressed_size),
                },
            })
        })
        .collect();

    Ok(GamePackageInfo {
        version: major.version,
        game_packages,
        audio_packages,
    })
}

pub async fn fetch_config(biz_id: &str, edition: HoyoEdition) -> Result<GameConfig> {
    let url = format!(
        "{}/getGameConfigs?launcher_id={}",
        edition.api_base(),
        edition.launcher_id()
    );

    let resp: ApiResponse<LaunchConfigsData> = reqwest::get(&url)
        .await
        .map_err(|e| anyhow!("failed to reach hoyo config api: {}", e))?
        .json()
        .await
        .map_err(|e| anyhow!("failed to parse hoyo config response: {}", e))?;

    if resp.retcode != 0 {
        return Err(anyhow!(
            "hoyo config api error {}: {}",
            resp.retcode,
            resp.message
        ));
    }

    let data = resp
        .data
        .ok_or_else(|| anyhow!("hoyo config api returned no data"))?;

    let config = data
        .launch_configs
        .into_iter()
        .find(|lc| lc.game.id == biz_id)
        .ok_or_else(|| anyhow!("game config {} not found in api response", biz_id))?;

    Ok(GameConfig {
        exe_name: config.exe_file_name,
        audio_pkg_path: config.audio_pkg_res_dir.unwrap_or_default(),
    })
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    retcode: i32,
    message: String,
    data: Option<T>,
}

#[derive(Deserialize)]
struct GamePackagesData {
    game_packages: Vec<GamePackageEntry>,
}

#[derive(Deserialize)]
struct GamePackageEntry {
    game: GameRef,
    main: MainVersion,
}

#[derive(Deserialize)]
struct GameRef {
    id: String,
}

#[derive(Deserialize)]
struct MainVersion {
    major: Option<MajorVersion>,
}

#[derive(Deserialize)]
struct MajorVersion {
    version: String,
    game_pkgs: Vec<RawPackage>,
    audio_pkgs: Vec<RawAudioPackage>,
}

#[derive(Deserialize)]
struct RawPackage {
    url: String,
    md5: String,
    size: String,
    decompressed_size: String,
}

#[derive(Deserialize)]
struct RawAudioPackage {
    language: String,
    url: String,
    md5: String,
    size: String,
    decompressed_size: String,
}

#[derive(Deserialize)]
struct LaunchConfigsData {
    launch_configs: Vec<LaunchConfigEntry>,
}

#[derive(Deserialize)]
struct LaunchConfigEntry {
    game: GameRef,
    exe_file_name: String,
    audio_pkg_res_dir: Option<String>,
}

fn parse_size(s: &str) -> u64 {
    s.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gacha::hoyo::{HoyoEdition, VoiceLocale};

    // genshin global biz_id, hardcoded for live test only
    const GENSHIN_GLOBAL: &str = "gopR6Cufr3";
    const HSR_GLOBAL: &str = "4ziysqXOQ8";

    #[tokio::test]
    #[ignore]
    async fn fetch_genshin_global_size_live() {
        let r =
            fetch_install_size(GENSHIN_GLOBAL, HoyoEdition::Global, &[VoiceLocale::English]).await;
        match r {
            Ok(s) => {
                println!("download={} install={}", s.download_bytes, s.install_bytes);
                assert!(s.install_bytes > 0, "install size should be non-zero");
            }
            Err(e) => panic!("fetch failed: {}", e),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn inspect_hsr_packages() {
        let info = fetch_packages(HSR_GLOBAL, HoyoEdition::Global)
            .await
            .unwrap();
        println!("version: {}", info.version);
        println!("game_packages ({}):", info.game_packages.len());
        for (i, p) in info.game_packages.iter().enumerate() {
            let fname = p.url.rsplit('/').next().unwrap_or(&p.url);
            println!(
                "  [{}] dl={:>12} inst={:>12}  {}",
                i, p.size, p.decompressed_size, fname
            );
        }
        println!("audio_packages ({}):", info.audio_packages.len());
        for (i, a) in info.audio_packages.iter().enumerate() {
            let fname = a.file.url.rsplit('/').next().unwrap_or(&a.file.url);
            println!(
                "  [{}] {:?} dl={:>12} inst={:>12}  {}",
                i, a.locale, a.file.size, a.file.decompressed_size, fname
            );
        }
    }

    #[tokio::test]
    #[ignore]
    async fn inspect_genshin_packages() {
        let info = fetch_packages(GENSHIN_GLOBAL, HoyoEdition::Global)
            .await
            .unwrap();
        println!("version: {}", info.version);
        println!("game_packages ({}):", info.game_packages.len());
        for (i, p) in info.game_packages.iter().enumerate() {
            let fname = p.url.rsplit('/').next().unwrap_or(&p.url);
            println!(
                "  [{}] dl={:>12} inst={:>12}  {}",
                i, p.size, p.decompressed_size, fname
            );
        }
        println!("audio_packages ({}):", info.audio_packages.len());
        for (i, a) in info.audio_packages.iter().enumerate() {
            let fname = a.file.url.rsplit('/').next().unwrap_or(&a.file.url);
            println!(
                "  [{}] {:?} dl={:>12} inst={:>12}  {}",
                i, a.locale, a.file.size, a.file.decompressed_size, fname
            );
        }
    }
}
