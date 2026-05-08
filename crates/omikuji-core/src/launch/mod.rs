use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::library::Game;

pub struct LaunchConfig {
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: PathBuf,
    pub game_id: String,
    pub game_name: String,
    pub pre_launch_script: String,
    pub post_exit_script: String,
}

// wine build variant, detected from version string
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WineVariant {
    System,
    WineGE,
    // proton requires umu-launcher
    Proton,
}

impl WineVariant {
    pub fn from_version(version: &str) -> Self {
        if version.is_empty() || version == "system" {
            WineVariant::System
        } else if version.starts_with("GE-Proton") || version.starts_with("Proton") || version.starts_with("dwproton") || version.starts_with("proton") {
            WineVariant::Proton
        } else if version.starts_with("steam:") {
            let steam_version = version.strip_prefix("steam:").unwrap_or(version);
            if steam_version.starts_with("GE-Proton") || steam_version.starts_with("Proton") || steam_version.starts_with("dwproton") || steam_version.starts_with("proton") {
                WineVariant::Proton
            } else {
                WineVariant::WineGE
            }
        } else {
            WineVariant::WineGE
        }
    }
}

pub fn build_launch(game: &Game) -> Result<LaunchConfig> {
    let working_dir = resolve_working_dir(game);

    match game.runner.runner_type.as_str() {
        "steam" => return build_steam_launch(game, working_dir),
        "flatpak" => return build_flatpak_launch(game, working_dir),
        _ => {}
    }

    let variant = WineVariant::from_version(&game.wine.version);
    let wine_exe = resolve_wine_exe(variant, &game.wine.version)?;
    let mut env = build_env(game, variant, &wine_exe);

    let mut command = if game.is_epic() {
        let legendary = crate::downloads::legendary::find_legendary()
            .ok_or_else(|| anyhow::anyhow!("legendary not found — required for Epic Games"))?;
        let prefix = resolve_prefix(game);
        // legendary wants the source app_id, falling back to metadata.id for games impoted before the source section existed
        let app_id = if !game.source.app_id.is_empty() {
            game.source.app_id.clone()
        } else {
            game.metadata.id.clone()
        };

        let mut cmd = vec![
            legendary.to_string_lossy().to_string(),
            "launch".to_string(),
            app_id,
            "--wine".to_string(),
            wine_exe.to_string_lossy().to_string(),
            "--wine-prefix".to_string(),
            prefix.to_string_lossy().to_string(),
            "--skip-version-check".to_string(),
        ];

        if !game.launch.args.is_empty() {
            cmd.push("--extra-args".to_string());
            cmd.push(game.launch.args.join(" "));
        }
        cmd
    } else {
        let mut cmd = vec![wine_exe.to_string_lossy().to_string()];
        if !game.metadata.exe.as_os_str().is_empty() {
            // jadeite spawns the game process itself, so extra args go after `--`
            if game.source.patch == "jadeite" {
                let jadeite_exe = crate::hoyo::jadeite_dir().join("jadeite.exe");
                cmd.push(jadeite_exe.to_string_lossy().to_string());
                cmd.push(game.metadata.exe.to_string_lossy().to_string());
                cmd.push("--".to_string());
            } else {
                cmd.push(game.metadata.exe.to_string_lossy().to_string());
            }
        }
        for arg in &game.launch.args {
            cmd.push(arg.clone());
        }
        cmd
    };

    apply_wrapping(&mut command, &mut env, game, true);

    Ok(LaunchConfig {
        command,
        env,
        working_dir,
        game_id: game.metadata.id.clone(),
        game_name: game.metadata.name.clone(),
        pre_launch_script: game.launch.pre_launch_script.clone(),
        post_exit_script: game.launch.post_exit_script.clone(),
    })
}

fn apply_wrapping(
    command: &mut Vec<String>,
    env: &mut HashMap<String, String>,
    game: &Game,
    wrap_mangohud: bool,
) {
    // mangohud only without gamescope; env var crashes with gamescope
    if wrap_mangohud && game.graphics.mangohud && !game.graphics.gamescope.enabled {
        command.insert(0, "mangohud".to_string());
        env.insert("MANGOHUD".to_string(), "1".to_string());
        env.insert("MANGOHUD_DLSYM".to_string(), "1".to_string());
    }

    if !game.launch.command_prefix.is_empty() {
        for (i, part) in game.launch.command_prefix.split_whitespace().enumerate() {
            command.insert(i, part.to_string());
        }
    }

    if game.system.cpu_limit > 0 {
        command.insert(0, format!("0-{}", game.system.cpu_limit - 1));
        command.insert(0, "-c".to_string());
        command.insert(0, "taskset".to_string());
    }

    if game.system.gamemode {
        command.insert(0, "gamemoderun".to_string());
    }

    if game.graphics.gamescope.enabled {
        let mut gs_cmd = vec!["gamescope".to_string()];
        gs_cmd.append(&mut build_gamescope_args(game));

        // mangohud with gamescope uses --mangoapp instead of env var
        if game.graphics.mangohud {
            gs_cmd.push("--mangoapp".to_string());
        }

        if game.graphics.gamescope.hdr {
            env.insert("DXVK_HDR".to_string(), "1".to_string());
        }

        gs_cmd.push("--".to_string());
        gs_cmd.append(command);
        *command = gs_cmd;
    }
}

fn effective_app_id(game: &Game) -> String {
    if !game.source.app_id.is_empty() {
        game.source.app_id.clone()
    } else {
        game.metadata.id.clone()
    }
}

fn build_steam_launch(game: &Game, working_dir: PathBuf) -> Result<LaunchConfig> {
    let appid = effective_app_id(game);
    if appid.is_empty() {
        anyhow::bail!("Steam runner requires an Application ID");
    }

    let steam_exe = find_steam_exe();
    let has_args = !game.launch.args.is_empty();

    // -applaunch needs the real steam binary; URL form falls back to xdg-open
    let mut command = if has_args {
        let exe = steam_exe.ok_or_else(|| anyhow::anyhow!(
            "Steam binary not found in PATH. Required for launching with arguments."
        ))?;
        let mut cmd = vec![exe, "-applaunch".to_string(), appid];
        for arg in &game.launch.args {
            cmd.push(arg.clone());
        }
        cmd
    } else {
        let exe = steam_exe.unwrap_or_else(|| "xdg-open".to_string());
        vec![exe, format!("steam://rungameid/{}", appid)]
    };

    let mut env: HashMap<String, String> = std::env::vars().collect();
    for (k, v) in &game.launch.env {
        env.insert(k.clone(), v.clone());
    }
    if game.system.pulse_latency {
        env.insert("PULSE_LATENCY_MSEC".to_string(), "60".to_string());
    }

    apply_wrapping(&mut command, &mut env, game, true);

    Ok(LaunchConfig {
        command,
        env,
        working_dir,
        game_id: game.metadata.id.clone(),
        game_name: game.metadata.name.clone(),
        pre_launch_script: game.launch.pre_launch_script.clone(),
        post_exit_script: game.launch.post_exit_script.clone(),
    })
}

fn build_flatpak_launch(game: &Game, working_dir: PathBuf) -> Result<LaunchConfig> {
    let appid = effective_app_id(game);
    if appid.is_empty() {
        anyhow::bail!("Flatpak runner requires an Application ID (e.g. org.foo.App)");
    }
    if appid.matches('.').count() < 2 {
        anyhow::bail!("Flatpak Application ID must look like tld.domain.app, got: {}", appid);
    }

    let mut command = vec!["flatpak".to_string(), "run".to_string()];

    // env table + mangohud + pulse_latency get translated to --env= flags so they reach inside the sandbox
    for (k, v) in &game.launch.env {
        command.push(format!("--env={}={}", k, v));
    }
    if game.graphics.mangohud && !game.graphics.gamescope.enabled {
        command.push("--env=MANGOHUD=1".to_string());
        command.push("--env=MANGOHUD_DLSYM=1".to_string());
    }
    if game.system.pulse_latency {
        command.push("--env=PULSE_LATENCY_MSEC=60".to_string());
    }

    command.push(appid);
    for arg in &game.launch.args {
        command.push(arg.clone());
    }

    let mut env: HashMap<String, String> = std::env::vars().collect();

    // mangohud is injected via --env above so the outer wrapper would double-set + leak into flatpak host process
    apply_wrapping(&mut command, &mut env, game, false);

    Ok(LaunchConfig {
        command,
        env,
        working_dir,
        game_id: game.metadata.id.clone(),
        game_name: game.metadata.name.clone(),
        pre_launch_script: game.launch.pre_launch_script.clone(),
        post_exit_script: game.launch.post_exit_script.clone(),
    })
}

fn build_gamescope_args(game: &Game) -> Vec<String> {
    let gs = &game.graphics.gamescope;
    let mut args = Vec::new();

    if gs.width > 0 {
        args.push("-W".to_string());
        args.push(gs.width.to_string());
    }
    if gs.height > 0 {
        args.push("-H".to_string());
        args.push(gs.height.to_string());
    }

    if gs.game_width > 0 {
        args.push("-w".to_string());
        args.push(gs.game_width.to_string());
    }
    if gs.game_height > 0 {
        args.push("-h".to_string());
        args.push(gs.game_height.to_string());
    }

    if gs.fps > 0 {
        args.push("-r".to_string());
        args.push(gs.fps.to_string());
    }

    if gs.fullscreen {
        args.push("-f".to_string());
    } else if gs.borderless {
        args.push("-b".to_string());
    }

    if gs.integer_scaling {
        args.push("-n".to_string());
    }

    if gs.hdr {
        args.push("--hdr-enabled".to_string());
    }

    if !gs.filter.is_empty() {
        args.push("-F".to_string());
        args.push(gs.filter.clone());
        if gs.filter == "fsr" && gs.fsr_sharpness > 0 {
            args.push("--fsr-sharpness".to_string());
            args.push(gs.fsr_sharpness.to_string());
        }
    }

    args
}

pub fn spawn(config: &LaunchConfig) -> Result<std::process::Child> {
    let mut cmd = Command::new(&config.command[0]);

    if config.command.len() > 1 {
        cmd.args(&config.command[1..]);
    }

    cmd.current_dir(&config.working_dir);
    cmd.env_clear();
    cmd.envs(&config.env);

    // detach from parent so the game keeps running if omikuji closes
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    let child = cmd.spawn()
        .with_context(|| format!("failed to spawn: {}", config.command[0]))?;

    Ok(child)
}

fn append_dll_override(env: &mut HashMap<String, String>, entry: &str) {
    let existing = env.get("WINEDLLOVERRIDES").map(|s| s.as_str()).unwrap_or("");
    let new_value = if existing.is_empty() {
        entry.to_string()
    } else {
        format!("{};{}", existing, entry)
    };
    env.insert("WINEDLLOVERRIDES".to_string(), new_value);
}

pub fn build_env(game: &Game, variant: WineVariant, wine_exe: &Path) -> HashMap<String, String> {
    let mut env = HashMap::new();

    for (k, v) in std::env::vars() {
        env.insert(k, v);
    }

    let prefix = resolve_prefix(game);
    env.insert("WINEPREFIX".to_string(), prefix.to_string_lossy().to_string());
    env.insert("WINEARCH".to_string(), game.wine.prefix_arch.clone());
    env.insert("WINE".to_string(), wine_exe.to_string_lossy().to_string());

    if variant == WineVariant::Proton {
        let proton_path = if game.wine.version.starts_with("steam:") {
            let steam_version = game.wine.version.strip_prefix("steam:").unwrap_or(&game.wine.version);
            crate::steam::local::resolve_or_default_proton(Some(steam_version))
                .unwrap_or_default()
        } else {
            runners_dir().join(&game.wine.version)
        };
        env.insert("PROTONPATH".to_string(), proton_path.to_string_lossy().to_string());
        env.insert("PROTON_VERB".to_string(), "run".to_string());
        env.insert("GAMEID".to_string(), game.metadata.id.clone());
    }

    env.insert("WINEESYNC".to_string(), if game.wine.esync { "1" } else { "0" }.to_string());
    env.insert("WINEFSYNC".to_string(), if game.wine.fsync { "1" } else { "0" }.to_string());

    if game.wine.dxvk {
        append_dll_override(&mut env, "d3d11,d3d10core,d3d9,d3d8,dxgi=n,b");
        env.insert("WINE_LARGE_ADDRESS_AWARE".to_string(), "1".to_string());
    }

    if game.wine.vkd3d {
        append_dll_override(&mut env, "d3d12=n,b");
    }

    if game.wine.dxvk_nvapi {
        env.insert("DXVK_ENABLE_NVAPI".to_string(), "1".to_string());
        env.insert("DXVK_NVAPIHACK".to_string(), "0".to_string());
        append_dll_override(&mut env, "nvapi,nvapi64=n,b");
    }

    if game.wine.battleye {
        env.insert("PROTON_BATTLEYE_RUNTIME".to_string(), "1".to_string());
    }
    if game.wine.easyanticheat {
        env.insert("PROTON_EAC_RUNTIME".to_string(), "1".to_string());
    }

    if game.wine.fsr {
        env.insert("WINE_FULLSCREEN_FSR".to_string(), "1".to_string());
    }

    if game.wine.audio_driver == "alsa" {
        append_dll_override(&mut env, "winepulse.drv=d");
    }

    if game.wine.graphics_driver == "wayland" {
        if variant == WineVariant::Proton {
            env.insert("PROTON_ENABLE_WAYLAND".to_string(), "1".to_string());
        } else {
            env.insert("DISPLAY".to_string(), String::new());
        }
    }

    if !game.wine.dll_overrides.is_empty() {
        let custom: Vec<String> = game.wine.dll_overrides.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        for entry in custom {
            append_dll_override(&mut env, &entry);
        }
    }

    if game.system.pulse_latency {
        env.insert("PULSE_LATENCY_MSEC".to_string(), "60".to_string());
    }

    if game.is_epic() {
        env.insert("LEGENDARY_WRAPPER_EXE".to_string(), "C:\\windows\\command\\EpicGamesLauncher.exe".to_string());
    }

    for (k, v) in &game.launch.env {
        env.insert(k.clone(), v.clone());
    }

    env
}

pub fn prepare_epic_prefix(game: &Game, wine_exe: &Path) -> Result<()> {
    let prefix = resolve_prefix(game);

    // spoof the epic launcher registry key so games that check for it dont bail early
    let mut cmd = std::process::Command::new(wine_exe);
    cmd.env("WINEPREFIX", prefix.to_string_lossy().to_string());
    cmd.args(["reg", "add", "HKEY_CLASSES_ROOT\\com.epicgames.launcher", "/f"]);
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());

    if let Err(e) = cmd.status() {
        eprintln!("[launch] epic registry spoof failed: {}", e);
    }

    let dummy_src = runtime_dir().join("EpicGamesLauncher.exe");
    if dummy_src.exists() {
        let dest_dir = prefix.join("drive_c").join("windows").join("command");
        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            eprintln!("[launch] failed to create command dir in prefix: {}", e);
        } else {
            let dest_file = dest_dir.join("EpicGamesLauncher.exe");
            if !dest_file.exists()
                && let Err(e) = std::fs::copy(&dummy_src, &dest_file) {
                    eprintln!("[launch] failed to copy dummy EpicGamesLauncher.exe: {}", e);
                }
        }
    }

    Ok(())
}

// for proton this returns umu-run, not wine; the actual proton path is set via PROTONPATH env in build_env
pub fn resolve_wine_exe(variant: WineVariant, version: &str) -> Result<PathBuf> {
    if version.starts_with("steam:") {
        let steam_version = version.strip_prefix("steam:").unwrap_or(version);
        return resolve_steam_runner(steam_version);
    }

    match variant {
        WineVariant::System => {
            Ok(PathBuf::from("wine"))
        }
        WineVariant::WineGE => {
            let runner_dir = runners_dir().join(version);
            let wine_bin = runner_dir.join("bin").join("wine");

            if wine_bin.exists() {
                Ok(wine_bin)
            } else {
                eprintln!("wine-ge not found: {}, falling back to system wine", wine_bin.display());
                Ok(PathBuf::from("wine"))
            }
        }
        WineVariant::Proton => {
            let umu_run = find_umu_run().ok_or_else(|| {
                anyhow::anyhow!(
                    "Proton requires umu-run (umu-launcher) but it's not found. \
                     Options:\n\
                     1. Install umu-run from your distro (package 'umu-launcher')\n\
                     2. Use Wine-GE instead of Proton-GE\n\
                     3. Copy Lutris's umu-run to ~/.local/share/omikuji/runtime/"
                )
            })?;

            let runner_dir = runners_dir().join(version);
            let proton_files = runner_dir.join("files");
            if !proton_files.exists() {
                anyhow::bail!("Proton not found at: {}", runner_dir.display());
            }

            Ok(umu_run)
        }
    }
}

fn resolve_steam_runner(version: &str) -> Result<PathBuf> {
    let install = crate::steam::local::resolve_or_default_proton(Some(version))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No Proton install available. Asked for '{}', couldn't find it or any fallback Proton in compatibilitytools.d / steamapps/common",
                version
            )
        })?;

    if install.join("files").exists() {
        let umu_run = find_umu_run()
            .ok_or_else(|| anyhow::anyhow!("umu-run required for Proton but not found"))?;
        return Ok(umu_run);
    }

    let wine_bin = install.join("bin").join("wine");
    if wine_bin.exists() {
        return Ok(wine_bin);
    }

    anyhow::bail!(
        "No valid wine/proton binary found in steam runner: {}",
        install.display()
    )
}

fn find_umu_run() -> Option<PathBuf> {
    let candidates = ["umu-run", "umu_run.py"];

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            for name in &candidates {
                let full_path = Path::new(dir).join(name);
                if full_path.exists() && is_executable(&full_path) {
                    return Some(full_path);
                }
            }
        }
    }

    let system_paths = [
        "/app/share/umu/umu-run",
        "/usr/share/umu/umu-run",
        "/usr/local/share/umu/umu-run",
        "/opt/umu/umu-run",
    ];

    for path in &system_paths {
        let p = Path::new(path);
        if p.exists() && is_executable(p) {
            return Some(p.to_path_buf());
        }
    }

    let our_runtime = runtime_dir().join("umu-run");
    if our_runtime.exists() && is_executable(&our_runtime) {
        return Some(our_runtime);
    }

    None
}

fn find_steam_exe() -> Option<String> {
    let candidates = ["steam", "steam.sh"];

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            for name in &candidates {
                let full_path = Path::new(dir).join(name);
                if full_path.exists() && is_executable(&full_path) {
                    return Some(name.to_string());
                }
            }
        }
    }

    let steam_paths = [
        "~/.steam/steam.sh",
        "~/.steam/steam/steam.sh",
        "~/.local/share/Steam/steam.sh",
        "~/.var/app/com.valvesoftware.Steam/data/Steam/steam.sh", // flatpak todooooootooTOODOOOOOOOOOOOO TODOOOOOOOOOO
    ];

    for path in &steam_paths {
        let expanded = shellexpand::tilde(path);
        let p = Path::new(expanded.as_ref());
        if p.exists() && is_executable(p) {
            return Some(p.to_string_lossy().to_string());
        }
    }

    None
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = std::fs::metadata(path) {
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    true
}

pub fn resolve_prefix(game: &Game) -> PathBuf {
    if !game.wine.prefix.is_empty() {
        return PathBuf::from(&game.wine.prefix);
    }

    let dir = prefixes_dir();

    // layout: prefixes/{slug}-{id}. if the name slugifies to nothing (e.g. non-ascii title) fall back to just the id so the dir is unique.
    let slug = if !game.metadata.slug.is_empty() {
        game.metadata.slug.clone()
    } else {
        crate::media::slugify(&game.metadata.name)
    };
    let folder = if slug.is_empty() {
        game.metadata.id.clone()
    } else {
        format!("{}-{}", slug, game.metadata.id)
    };
    let prefix = dir.join(folder);
    if !prefix.exists()
        && let Err(e) = std::fs::create_dir_all(&prefix) {
            eprintln!("failed to create prefix dir: {}", e);
        }
    prefix
}

fn resolve_working_dir(game: &Game) -> PathBuf {
    if game.launch.working_dir.is_empty() {
        game.metadata.exe.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(&game.launch.working_dir)
    }
}

fn runners_dir() -> PathBuf {
    crate::runners_dir()
}

fn prefixes_dir() -> PathBuf {
    crate::prefixes_dir()
}

fn runtime_dir() -> PathBuf {
    crate::runtime_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wine_variant_from_version() {
        assert_eq!(WineVariant::from_version(""), WineVariant::System);
        assert_eq!(WineVariant::from_version("system"), WineVariant::System);
        assert_eq!(WineVariant::from_version("wine-ge-9-5"), WineVariant::WineGE);
        assert_eq!(WineVariant::from_version("lutris-7.2"), WineVariant::WineGE);
        assert_eq!(WineVariant::from_version("GE-Proton10-34"), WineVariant::Proton);
        assert_eq!(WineVariant::from_version("Proton-9-0-4"), WineVariant::Proton);
    }

}
