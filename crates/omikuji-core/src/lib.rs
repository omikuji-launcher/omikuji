
pub mod archive_source;
pub mod component_state;
pub mod components;
pub mod defaults;
pub mod desktop;
pub mod discord;
pub mod dll_packs;
pub mod downloads;
pub mod endfield;
pub mod epic;
pub mod external;
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
pub mod notifications;
pub mod process;
pub mod runners;
pub mod settings;
pub mod steam;
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

pub fn runners_dir() -> PathBuf {
    settings::expand(&settings::get().paths.runners_dir)
}

pub fn dll_packs_dir() -> PathBuf {
    settings::expand(&settings::get().paths.dll_packs_dir)
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
