use super::spec::{ComponentSpec, ExtractStrategy, SettingsKey, Source};
use crate::archive::ArchiveKind;
use crate::launch::umu_system_path;
use crate::store::epic::source::legendary_system_path;

pub fn all() -> &'static [ComponentSpec] {
    COMPONENTS
}

static COMPONENTS: &[ComponentSpec] = &[
    ComponentSpec {
        name: "umu-run",
        source: Source::GithubRelease {
            asset_matcher: |n| n.ends_with("-zipapp.tar"),
        },
        extract: ExtractStrategy::Archive {
            kind: ArchiveKind::Tar,
            inner_path: "umu-run",
        },
        dest: "umu-run",
        settings_key: SettingsKey::UmuRun,
        system_probe: Some(umu_system_path),
    },
    ComponentSpec {
        name: "hpatchz",
        source: Source::GithubRelease {
            asset_matcher: |n| n.contains("linux64") && n.ends_with(".zip"),
        },
        extract: ExtractStrategy::Archive {
            kind: ArchiveKind::Zip,
            inner_path: "hpatchz",
        },
        dest: "hpatchz",
        settings_key: SettingsKey::Hpatchz,
        system_probe: None,
    },
    ComponentSpec {
        name: "legendary",
        source: Source::GithubRelease {
            asset_matcher: |n| n.contains("linux") && n.contains("x64"),
        },
        extract: ExtractStrategy::Raw,
        dest: "legendary",
        settings_key: SettingsKey::Legendary,
        system_probe: Some(legendary_system_path),
    },
    ComponentSpec {
        name: "gogdl",
        source: Source::GithubRelease {
            asset_matcher: |n| n == "gogdl_linux_x86_64",
        },
        extract: ExtractStrategy::Raw,
        dest: "gogdl",
        settings_key: SettingsKey::Gogdl,
        system_probe: Some(|| which::which("gogdl").ok()),
    },
    ComponentSpec {
        name: "nile",
        source: Source::GithubRelease {
            asset_matcher: |n| n.contains("linux") && n.contains("x86_64"),
        },
        extract: ExtractStrategy::Raw,
        dest: "nile",
        settings_key: SettingsKey::Nile,
        system_probe: Some(|| which::which("nile").ok()),
    },
    ComponentSpec {
        name: "egl-dummy",
        source: Source::DirectUrl { marker: "bundled" },
        extract: ExtractStrategy::Raw,
        dest: "EpicGamesLauncher.exe",
        settings_key: SettingsKey::EglDummy,
        system_probe: None,
    },
];
