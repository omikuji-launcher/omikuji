pub mod api;
pub mod installer;
pub mod manifest;
pub mod patcher;

pub mod protos {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/sophon.rs"));
}

use super::HoyoEdition;

pub fn branches_host(edition: HoyoEdition) -> &'static str {
    match edition {
        HoyoEdition::Global => "https://sg-hyp-api.hoyoverse.com",
        HoyoEdition::China => "https://hyp-api.mihoyo.com",
    }
}

// different host from branches_host, not a typo, maybe
pub fn api_host(edition: HoyoEdition) -> &'static str {
    match edition {
        HoyoEdition::Global => "https://sg-public-api.hoyoverse.com",
        HoyoEdition::China => "https://api-takumi.mihoyo.com",
    }
}

pub fn game_branches_url(edition: HoyoEdition) -> String {
    format!(
        "{}/hyp/hyp-connect/api/getGameBranches?launcher_id={}",
        branches_host(edition),
        edition.launcher_id()
    )
}

pub fn patch_build_url(edition: HoyoEdition, pkg: &api::PackageInfo) -> String {
    format!(
        "{}/downloader/sophon_chunk/api/getPatchBuild?branch={}&password={}&package_id={}",
        api_host(edition),
        pkg.branch,
        pkg.password,
        pkg.package_id,
    )
}

pub fn build_url(edition: HoyoEdition, pkg: &api::PackageInfo) -> String {
    format!(
        "{}/downloader/sophon_chunk/api/getBuild?branch={}&password={}&package_id={}",
        api_host(edition),
        pkg.branch,
        pkg.password,
        pkg.package_id,
    )
}
