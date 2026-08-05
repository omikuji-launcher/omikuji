use super::spec::{ComponentSpec, ExtractStrategy, SettingsKey, Source, Trigger};

pub fn all() -> &'static [ComponentSpec] {
    COMPONENTS
}

static COMPONENTS: &[ComponentSpec] = &[
    ComponentSpec {
        name: "umu-run",
        source: Source::GithubRelease {
            asset_matcher: |n| n.ends_with("-zipapp.tar"),
        },
        extract: ExtractStrategy::Tar {
            inner_path: "umu-run",
        },
        dest: "umu-run",
        settings_key: SettingsKey::UmuRun,
        trigger: Trigger::Eager,
    },
    ComponentSpec {
        name: "hpatchz",
        source: Source::GithubRelease {
            asset_matcher: |n| n.contains("linux64") && n.ends_with(".zip"),
        },
        extract: ExtractStrategy::Zip {
            inner_path: "hpatchz",
        },
        dest: "hpatchz",
        settings_key: SettingsKey::Hpatchz,
        trigger: Trigger::OnDemand,
    },
    ComponentSpec {
        name: "legendary",
        source: Source::GithubRelease {
            asset_matcher: |n| n.contains("linux") && n.contains("x64"),
        },
        extract: ExtractStrategy::Raw,
        dest: "legendary",
        settings_key: SettingsKey::Legendary,
        trigger: Trigger::OnDemand,
    },
    ComponentSpec {
        name: "gogdl",
        source: Source::GithubRelease {
            asset_matcher: |n| n == "gogdl_linux_x86_64",
        },
        extract: ExtractStrategy::Raw,
        dest: "gogdl",
        settings_key: SettingsKey::Gogdl,
        trigger: Trigger::OnDemand,
    },
    ComponentSpec {
        name: "egl-dummy",
        source: Source::DirectUrl { marker: "bundled" },
        extract: ExtractStrategy::Raw,
        dest: "EpicGamesLauncher.exe",
        settings_key: SettingsKey::EglDummy,
        trigger: Trigger::OnDemand,
    },
];
