pub use anyhow;

pub mod archive_source;
pub mod components;
pub mod components_config;
pub mod defaults;
pub mod desktop;
pub mod discord;
pub mod dll_packs;
pub mod downloads;
pub mod endfield;
pub mod epic;
pub mod external;
pub mod fs_util;
pub mod fs_watcher;
pub mod gachas;
pub mod gog;
pub mod game_logs;
pub mod hoyo;
pub mod install_sizes;
pub mod kuro;
pub mod launch;
pub mod library;
pub mod media;
pub mod migration;
pub mod notifications;
pub mod prefixes;
pub mod process;
pub mod runners;
pub mod settings;
pub mod steam;
pub mod system_info;
pub mod ui_settings;
pub mod wine_tools;

use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    settings::expand(&settings::get().paths.data_dir)
}

pub fn library_dir() -> PathBuf {
    settings::expand(&settings::get().paths.library_dir)
}

pub fn gachas_dir() -> PathBuf {
    settings::expand(&settings::get().paths.gachas_dir)
}

pub fn components_dir() -> PathBuf {
    settings::expand(&settings::get().paths.components_dir)
}

pub fn runners_dir() -> PathBuf {
    components_subdir(&settings::get().paths.runners_dir, "runners")
}

pub fn layers_dir() -> PathBuf {
    components_subdir(&settings::get().paths.layers_dir, "layers")
}

fn components_subdir(override_path: &str, sub: &str) -> PathBuf {
    if override_path.is_empty() {
        components_dir().join(sub)
    } else {
        settings::expand(override_path)
    }
}

pub fn prefixes_dir() -> PathBuf {
    settings::expand(&settings::get().paths.prefixes_dir)
}

pub fn cache_dir() -> PathBuf {
    settings::expand(&settings::get().paths.cache_dir)
}

// logs live under cache so periodic cleanup reclaims space without touching load-bearing state
pub fn logs_dir() -> PathBuf {
    cache_dir().join("logs")
}

pub fn runtime_dir() -> PathBuf {
    settings::expand(&settings::get().paths.runtime_dir)
}
