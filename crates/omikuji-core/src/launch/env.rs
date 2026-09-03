use std::collections::HashMap;
use std::path::Path;

use super::prefix::resolve_prefix;
use super::wine::{ProtonVerb, WineVariant};
use crate::library::Game;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvPurpose {
    Session,
    Tool,
}

const BATTLEYE_RUNTIME_APPID: &str = "1161040";
const EAC_RUNTIME_APPID: &str = "1826330";

pub fn build_env(
    game: &Game,
    variant: WineVariant,
    wine_exe: &Path,
    purpose: EnvPurpose,
) -> HashMap<String, String> {
    let mut env = HashMap::new();

    for (k, v) in std::env::vars() {
        env.insert(k, v);
    }

    env.insert("WINEDEBUG".to_string(), String::new());

    let prefix = resolve_prefix(game);
    env.insert(
        "WINEPREFIX".to_string(),
        prefix.to_string_lossy().to_string(),
    );
    env.insert("WINEARCH".to_string(), game.wine.prefix_arch.clone());
    env.insert("WINE".to_string(), wine_exe.to_string_lossy().to_string());

    if variant == WineVariant::Proton {
        let proton_path = if game.wine.version.starts_with("steam:") {
            let steam_version = game
                .wine
                .version
                .strip_prefix("steam:")
                .unwrap_or(&game.wine.version);
            crate::store::steam::local::resolve_or_default_proton(Some(steam_version))
                .unwrap_or_default()
        } else {
            crate::runners::installed_runner_dir(&game.wine.version)
                .unwrap_or_else(|| crate::runners_dir().join(&game.wine.version))
        };
        env.insert(
            "PROTONPATH".to_string(),
            proton_path.to_string_lossy().to_string(),
        );
        env.insert(
            "PROTON_VERB".to_string(),
            ProtonVerb::Run.as_str().to_string(),
        );
        env.insert(
            "GAMEID".to_string(),
            format!(
                "umu-{}",
                crate::store::steam::synthetic_appid(&game.metadata.id)
            ),
        );
    }

    env.insert(
        "WINEESYNC".to_string(),
        if game.wine.esync { "1" } else { "0" }.to_string(),
    );
    env.insert(
        "WINEFSYNC".to_string(),
        if game.wine.fsync { "1" } else { "0" }.to_string(),
    );

    if variant == WineVariant::Proton {
        let ntsync = game.wine.ntsync;
        env.insert(
            "PROTON_USE_NTSYNC".to_string(),
            if ntsync { "1" } else { "0" }.to_string(),
        );

        // Proton 11+ uses PROTON_NO_NTSYNC to disable NTSync as seen in cachyos-proton 11.0-20260428
        env.insert(
            "PROTON_NO_NTSYNC".to_string(),
            if ntsync { "0" } else { "1" }.to_string(),
        );
    }

    if game.wine.dxvk {
        append_dll_override(&mut env, "d3d11,d3d10core,d3d9,d3d8,dxgi=n,b");
        env.insert("WINE_LARGE_ADDRESS_AWARE".to_string(), "1".to_string());
    }

    if game.wine.vkd3d {
        append_dll_override(&mut env, "d3d12,d3d12core=n,b");
    }

    if game.wine.dxvk_nvapi {
        env.insert("DXVK_ENABLE_NVAPI".to_string(), "1".to_string());
        env.insert("DXVK_NVAPIHACK".to_string(), "0".to_string());
        append_dll_override(&mut env, "nvapi,nvapi64=n,b");
    }

    if variant == WineVariant::Proton {
        if game.wine.battleye
            && let Some(dir) = anticheat_runtime(BATTLEYE_RUNTIME_APPID)
        {
            env.insert("PROTON_BATTLEYE_RUNTIME".to_string(), dir);
        }
        if game.wine.easyanticheat
            && let Some(dir) = anticheat_runtime(EAC_RUNTIME_APPID)
        {
            env.insert("PROTON_EAC_RUNTIME".to_string(), dir);
        }
    }

    if game.wine.fsr {
        env.insert("WINE_FULLSCREEN_FSR".to_string(), "1".to_string());
    }

    if game.wine.audio_driver == "alsa" {
        append_dll_override(&mut env, "winepulse.drv=d");
    }

    if purpose == EnvPurpose::Session && game.wine.graphics_driver == "wayland" {
        if variant == WineVariant::Proton {
            env.insert("PROTON_ENABLE_WAYLAND".to_string(), "1".to_string());
        } else {
            env.insert("DISPLAY".to_string(), String::new());
        }
    }

    if !game.wine.dll_overrides.is_empty() {
        let custom: Vec<String> = game
            .wine
            .dll_overrides
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        for entry in custom {
            append_dll_override(&mut env, &entry);
        }
    }

    if !game.wine.dll_override_sets.is_empty() {
        let ui = crate::app_settings::AppSettings::load();
        apply_kv_sets(&ui.dll_sets, &game.wine.dll_override_sets, |key, value| {
            append_dll_override(&mut env, &format!("{key}={value}"));
        });
    }

    if game.is_epic() {
        env.insert(
            "LEGENDARY_WRAPPER_EXE".to_string(),
            "C:\\windows\\command\\EpicGamesLauncher.exe".to_string(),
        );
    }

    env.extend(game_env_pairs(game));

    env
}

fn apply_kv_sets(
    sets: &[crate::app_settings::KvSet],
    ids: &[String],
    mut apply: impl FnMut(&str, &str),
) {
    for id in ids {
        let Some(set) = sets.iter().find(|s| &s.id == id) else {
            continue;
        };
        for pair in &set.vars {
            if !pair.key.trim().is_empty() {
                apply(&pair.key, &pair.value);
            }
        }
    }
}

pub(super) fn game_env_pairs(game: &Game) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    if game.system.pulse_latency {
        pairs.push(("PULSE_LATENCY_MSEC".to_string(), "60".to_string()));
    }
    for (k, v) in &game.launch.env {
        pairs.push((k.clone(), v.clone()));
    }
    if !game.launch.env_sets.is_empty() {
        let ui = crate::app_settings::AppSettings::load();
        apply_kv_sets(&ui.env_sets, &game.launch.env_sets, |key, value| {
            pairs.push((key.to_string(), value.to_string()));
        });
    }
    pairs
}

// a depot husk keeps the dir but loses the v* payload proton's ntdll loads from
fn anticheat_runtime(appid: &str) -> Option<String> {
    let dir = crate::store::steam::local::get_game_install_dir(appid)?;
    let has_payload = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .any(|entry| entry.file_name().to_string_lossy().starts_with('v') && entry.path().is_dir());
    has_payload.then(|| dir.to_string_lossy().into_owned())
}

fn append_dll_override(env: &mut HashMap<String, String>, entry: &str) {
    let existing = env
        .get("WINEDLLOVERRIDES")
        .map(|s| s.as_str())
        .unwrap_or("");
    let new_value = if existing.is_empty() {
        entry.to_string()
    } else {
        format!("{};{}", existing, entry)
    };
    env.insert("WINEDLLOVERRIDES".to_string(), new_value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn game(version: &str, ntsync: bool) -> Game {
        let mut game = Game::new("Test".to_string(), PathBuf::from("/tmp/test.exe"));
        game.wine.version = version.to_string();
        game.wine.ntsync = ntsync;
        game
    }

    #[test]
    fn test_ntsync_env_for_proton() {
        let enabled = build_env(
            &game("Proton-9-0-4", true),
            WineVariant::Proton,
            Path::new("wine"),
            EnvPurpose::Session,
        );
        assert_eq!(
            enabled.get("PROTON_USE_NTSYNC").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            enabled.get("PROTON_NO_NTSYNC").map(String::as_str),
            Some("0")
        );

        let disabled = build_env(
            &game("Proton-9-0-4", false),
            WineVariant::Proton,
            Path::new("wine"),
            EnvPurpose::Session,
        );
        assert_eq!(
            disabled.get("PROTON_USE_NTSYNC").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            disabled.get("PROTON_NO_NTSYNC").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn test_ntsync_env_not_added_for_non_proton() {
        let inherited_use = std::env::var_os("PROTON_USE_NTSYNC").is_some();
        let inherited_no = std::env::var_os("PROTON_NO_NTSYNC").is_some();
        let env = build_env(
            &game("wine-ge-9-5", true),
            WineVariant::Runner,
            Path::new("wine"),
            EnvPurpose::Session,
        );

        if !inherited_use {
            assert!(!env.contains_key("PROTON_USE_NTSYNC"));
        }
        if !inherited_no {
            assert!(!env.contains_key("PROTON_NO_NTSYNC"));
        }
    }

    #[test]
    fn test_anticheat_env_not_added_for_non_proton() {
        let inherited_be = std::env::var_os("PROTON_BATTLEYE_RUNTIME").is_some();
        let inherited_eac = std::env::var_os("PROTON_EAC_RUNTIME").is_some();
        let mut game = game("wine-ge-9-5", false);
        game.wine.battleye = true;
        game.wine.easyanticheat = true;
        let env = build_env(
            &game,
            WineVariant::Runner,
            Path::new("wine"),
            EnvPurpose::Session,
        );

        if !inherited_be {
            assert!(!env.contains_key("PROTON_BATTLEYE_RUNTIME"));
        }
        if !inherited_eac {
            assert!(!env.contains_key("PROTON_EAC_RUNTIME"));
        }
    }
}
