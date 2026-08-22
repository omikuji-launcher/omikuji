use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ComponentSpec {
    pub name: &'static str,
    pub source: Source,
    pub extract: ExtractStrategy,
    pub dest: &'static str,
    pub settings_key: SettingsKey,
    pub system_probe: Option<fn() -> Option<PathBuf>>,
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsKey {
    UmuRun,
    Hpatchz,
    Legendary,
    Gogdl,
    EglDummy,
}

#[derive(Debug, Clone, Copy)]
pub enum Source {
    GithubRelease { asset_matcher: fn(&str) -> bool },
    // marker is a static sentinel so teh version check can distinguish installed vs missing
    DirectUrl { marker: &'static str },
}

#[derive(Debug, Clone, Copy)]
pub enum ExtractStrategy {
    Raw,
    Tar { inner_path: &'static str },
    TarGz { inner_path: &'static str },
    Zip { inner_path: &'static str },
}

#[derive(Debug, Clone)]
pub enum ComponentStatus {
    Installed { version: String, path: PathBuf },
    System { path: PathBuf },
    Missing,
}
