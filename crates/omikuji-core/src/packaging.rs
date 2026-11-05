use crate::fs_util::find_executable_in_paths;
use std::env;
use std::path::Path;

pub fn in_flatpak() -> bool {
    env::var("FLATPAK_ID").is_ok() || Path::new("/.flatpak-info").exists()
}

pub enum Packaging {
    Flatpak { app_id: String },
    AppImage { path: String },
    Native,
}

impl Packaging {
    pub fn detect() -> Self {
        if let Ok(app_id) = env::var("FLATPAK_ID") {
            Self::Flatpak { app_id }
        } else if let Ok(path) = env::var("APPIMAGE") {
            Self::AppImage { path }
        } else {
            Self::Native
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Flatpak { .. } => "flatpak",
            Self::AppImage { .. } => "appimage",
            Self::Native => "native",
        }
    }

    pub fn host_invocation(&self) -> Vec<String> {
        match self {
            Self::Flatpak { app_id } => {
                vec!["/usr/bin/flatpak".into(), "run".into(), app_id.clone()]
            }
            Self::AppImage { path } => vec![path.clone()],
            Self::Native => vec![native_exe()],
        }
    }
}

// PATH first since current_exe could be some nix store path (not sure it breaks tho)
fn native_exe() -> String {
    find_executable_in_paths(&["omikuji"], &[])
        .or_else(|| env::current_exe().ok())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "omikuji".into())
}
