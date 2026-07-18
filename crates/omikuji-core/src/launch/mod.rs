use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::library::Game;

use crate::template_vars::TemplateVars;

#[derive(Debug)]
pub struct ComponentMissing {
    pub name: String,
}

impl std::fmt::Display for ComponentMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` not found", self.name)
    }
}

impl std::error::Error for ComponentMissing {}

pub struct LaunchConfig {
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: PathBuf,
    pub game_id: String,
    pub game_name: String,
    pub post_exit_script: String,
}

impl LaunchConfig {
    fn from_game(
        game: &Game,
        command: Vec<String>,
        env: HashMap<String, String>,
        working_dir: PathBuf,
    ) -> Self {
        let vars = TemplateVars::for_game(game);
        Self {
            command: command.into_iter().map(|c| vars.expand(&c)).collect(),
            env: vars.expand_env(env),
            working_dir: PathBuf::from(vars.expand(&working_dir.to_string_lossy())),
            game_id: game.metadata.id.clone(),
            game_name: game.metadata.name.clone(),
            post_exit_script: vars.expand(&game.launch.post_exit_script),
        }
    }
}

// wine build variant, detected from version string
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WineVariant {
    System,
    WineGE,
    // proton requires umu-launcher
    Proton,
}

fn looks_like_proton(s: &str) -> bool {
    s.starts_with("GE-Proton")
        || s.starts_with("Proton")
        || s.starts_with("dwproton")
        || s.starts_with("proton")
}

impl WineVariant {
    pub fn from_version(version: &str) -> Self {
        if version.is_empty() || version == "system" {
            return WineVariant::System;
        }
        let name = version.strip_prefix("steam:");
        let dir = match name {
            Some(rest) => crate::steam::local::find_proton_install(rest),
            None => crate::runners::installed_runner_dir(version),
        };
        match dir {
            Some(dir) if crate::runners::is_proton_dir(&dir) => WineVariant::Proton,
            Some(_) => WineVariant::WineGE,
            None if looks_like_proton(name.unwrap_or(version)) => WineVariant::Proton,
            None => WineVariant::WineGE,
        }
    }
}

pub fn prepare_launch(game: &Game) -> Result<LaunchConfig> {
    let config = assemble_launch(game)?;
    reject_slop_env(&config)?;
    run_pre_launch_script(game, &config);
    validate_exe(game)?;
    Ok(config)
}

pub fn build_launch(game: &Game) -> Result<LaunchConfig> {
    let config = assemble_launch(game)?;
    reject_slop_env(&config)?;
    validate_exe(game)?;
    Ok(config)
}

fn reject_slop_env(config: &LaunchConfig) -> Result<()> {
    if config.env.contains_key("WINE_CANONICAL_HOLE") {
        anyhow::bail!(
            "WINE_CANONICAL_HOLE detected in the launch environment. bro remove this shit pls. this variable is not real, wine has no canonical hole, and whatever slop config it came from probably broke other things too :xdd:"
        );
    }
    Ok(())
}

fn run_pre_launch_script(game: &Game, config: &LaunchConfig) {
    let script = &TemplateVars::for_game(game).expand(&game.launch.pre_launch_script);
    if script.is_empty() {
        return;
    }
    tracing::info!("running pre-launch script: {}", script);
    let cwd = if config.working_dir.exists() {
        config.working_dir.clone()
    } else {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    };
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(script.as_str())
        .current_dir(&cwd)
        .envs(&config.env)
        .status();
    match status {
        Ok(s) if !s.success() => tracing::warn!("pre-launch script exited with: {}", s),
        Err(e) => tracing::error!("pre-launch script failed: {}", e),
        _ => {}
    }
}

fn validate_exe(game: &Game) -> Result<()> {
    let exe = &game.metadata.exe;
    match game.runner.runner_type.as_str() {
        "steam" | "flatpak" => Ok(()),
        "native" => {
            if exe.as_os_str().is_empty() {
                anyhow::bail!("Native runner requires an executable");
            }
            if !exe.exists() {
                anyhow::bail!("Game executable not found at `{}`", exe.display());
            }
            if !is_executable(exe) {
                anyhow::bail!(
                    "`{}` is not executable. Mark it executable (chmod +x) and try again.",
                    exe.display()
                );
            }
            Ok(())
        }
        _ => {
            if !game.is_epic() && !exe.as_os_str().is_empty() && !exe.exists() {
                anyhow::bail!("Game executable not found at `{}`", exe.display());
            }
            Ok(())
        }
    }
}

fn assemble_launch(game: &Game) -> Result<LaunchConfig> {
    let working_dir = resolve_working_dir(game);

    match game.runner.runner_type.as_str() {
        "steam" => return build_steam_launch(game, working_dir),
        "flatpak" => return build_flatpak_launch(game, working_dir),
        "native" => return build_native_launch(game, working_dir),
        _ => {}
    }

    let variant = WineVariant::from_version(&game.wine.version);
    let wine_exe = resolve_wine_exe(variant, &game.wine.version)?;
    let mut env = build_env(game, variant, &wine_exe, EnvPurpose::Session);

    if variant == WineVariant::Proton
        && let Err(e) = crate::desktop::ensure_steam_icon(game)
    {
        tracing::warn!("dock icon link failed for {}: {}", game.metadata.name, e);
    }

    let mut command = if game.is_epic() {
        let legendary = crate::downloads::legendary::find_legendary().ok_or_else(|| {
            anyhow::Error::new(ComponentMissing {
                name: "Legendary".to_string(),
            })
        })?;
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

    Ok(LaunchConfig::from_game(game, command, env, working_dir))
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

    if wrap_mangohud {
        for (k, v) in crate::system_info::gpu_launch_env(&game.graphics.gpu) {
            env.insert(k, v);
        }
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

    let mut command = build_steam_command(&appid, &game.launch.args);

    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.extend(game_env_pairs(game));

    apply_wrapping(&mut command, &mut env, game, true);

    Ok(LaunchConfig::from_game(game, command, env, working_dir))
}

fn build_flatpak_launch(game: &Game, working_dir: PathBuf) -> Result<LaunchConfig> {
    let appid = effective_app_id(game);
    if appid.is_empty() {
        anyhow::bail!("Flatpak runner requires an Application ID (e.g. org.foo.App)");
    }
    if appid.matches('.').count() < 2 {
        anyhow::bail!(
            "Flatpak Application ID must look like tld.domain.app, got: {}",
            appid
        );
    }

    let mut command = vec!["flatpak".to_string(), "run".to_string()];

    // game env + mangohud get translated to --env= flags so they reach inside the sandbox
    for (k, v) in game_env_pairs(game) {
        command.push(format!("--env={}={}", k, v));
    }
    if game.graphics.mangohud && !game.graphics.gamescope.enabled {
        command.push("--env=MANGOHUD=1".to_string());
        command.push("--env=MANGOHUD_DLSYM=1".to_string());
    }
    for (k, v) in crate::system_info::gpu_launch_env(&game.graphics.gpu) {
        command.push(format!("--env={}={}", k, v));
    }

    command.push(appid);
    for arg in &game.launch.args {
        command.push(arg.clone());
    }

    let mut env: HashMap<String, String> = std::env::vars().collect();

    // mangohud is injected via --env above so the outer wrapper would double-set + leak into flatpak host process
    apply_wrapping(&mut command, &mut env, game, false);

    Ok(LaunchConfig::from_game(game, command, env, working_dir))
}

fn build_native_launch(game: &Game, working_dir: PathBuf) -> Result<LaunchConfig> {
    let exe = &game.metadata.exe;
    let mut command = vec![relative_exe(exe, &working_dir)];
    for arg in &game.launch.args {
        command.push(arg.clone());
    }

    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.extend(game_env_pairs(game));

    apply_wrapping(&mut command, &mut env, game, true);

    Ok(LaunchConfig::from_game(game, command, env, working_dir))
}

// we trust lutris with this one guys
fn relative_exe(exe: &Path, working_dir: &Path) -> String {
    match exe.strip_prefix(working_dir) {
        Ok(rel) => format!("./{}", rel.display()),
        Err(_) => exe.to_string_lossy().to_string(),
    }
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

    if gs.refresh_rate > 0 {
        args.push("-r".to_string());
        args.push(gs.refresh_rate.to_string());
    }

    if gs.fps > 0 {
        args.push("--framerate-limit".to_string());
        args.push(gs.fps.to_string());
    }

    if gs.fullscreen {
        args.push("-f".to_string());
    } else if gs.borderless {
        args.push("-b".to_string());
    }

    if gs.integer_scaling {
        args.push("-S".to_string());
        args.push("integer".to_string());
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

    let child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn: {}", config.command[0]))?;

    Ok(child)
}

fn apply_kv_sets(
    sets: &[crate::ui_settings::KvSet],
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

fn game_env_pairs(game: &Game) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    if game.system.pulse_latency {
        pairs.push(("PULSE_LATENCY_MSEC".to_string(), "60".to_string()));
    }
    for (k, v) in &game.launch.env {
        pairs.push((k.clone(), v.clone()));
    }
    if !game.launch.env_sets.is_empty() {
        let ui = crate::ui_settings::UiSettings::load();
        apply_kv_sets(&ui.env_sets, &game.launch.env_sets, |key, value| {
            pairs.push((key.to_string(), value.to_string()));
        });
    }
    pairs
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvPurpose {
    Session,
    Tool,
}

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
            crate::steam::local::resolve_or_default_proton(Some(steam_version)).unwrap_or_default()
        } else {
            crate::runners::installed_runner_dir(&game.wine.version)
                .unwrap_or_else(|| crate::runners_dir().join(&game.wine.version))
        };
        env.insert(
            "PROTONPATH".to_string(),
            proton_path.to_string_lossy().to_string(),
        );
        env.insert("PROTON_VERB".to_string(), "run".to_string());
        env.insert(
            "GAMEID".to_string(),
            format!("umu-{}", crate::steam::synthetic_appid(&game.metadata.id)),
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
        let ui = crate::ui_settings::UiSettings::load();
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

pub fn prepare_epic_prefix(
    game: &Game,
    wine_exe: &Path,
    env: &HashMap<String, String>,
) -> Result<()> {
    let prefix = resolve_prefix(game);

    // spoof the epic launcher registry key so games that check for it dont bail early
    let mut cmd = Command::new(wine_exe);
    cmd.env_clear();
    cmd.envs(env);
    if WineVariant::from_version(&game.wine.version) == WineVariant::Proton {
        cmd.env("PROTON_VERB", "waitforexitandrun");
    }
    cmd.args([
        "reg",
        "add",
        "HKEY_CLASSES_ROOT\\com.epicgames.launcher",
        "/f",
    ]);
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    if let Err(e) = cmd.status() {
        tracing::error!("epic registry spoof failed: {}", e);
    }

    let dummy_src = runtime_dir().join("EpicGamesLauncher.exe");
    if dummy_src.exists() {
        let dest_dir = prefix.join("drive_c").join("windows").join("command");
        if let Err(e) = std::fs::create_dir_all(&dest_dir) {
            tracing::error!("failed to create command dir in prefix: {}", e);
        } else {
            let dest_file = dest_dir.join("EpicGamesLauncher.exe");
            if !dest_file.exists()
                && let Err(e) = std::fs::copy(&dummy_src, &dest_file)
            {
                tracing::error!("failed to copy dummy EpicGamesLauncher.exe: {}", e);
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

    if let Some(name) = version.strip_prefix("system:") {
        if let Some(path) = crate::runners::system_wine_paths().get(name) {
            return Ok(path.clone());
        }
        anyhow::bail!("Runner `{}` not found.", name);
    }

    match variant {
        WineVariant::System => Ok(PathBuf::from("wine")),
        WineVariant::WineGE => crate::runners::installed_runner_dir(version)
            .map(|d| d.join("bin").join("wine"))
            .filter(|p| p.exists())
            .ok_or_else(|| anyhow::anyhow!("Runner `{}` not found.", version)),
        WineVariant::Proton => {
            let umu_run = find_umu_run().ok_or_else(|| {
                anyhow::Error::new(ComponentMissing {
                    name: "umu-run".to_string(),
                })
            })?;

            let has_files = crate::runners::installed_runner_dir(version)
                .map(|d| d.join("files").exists())
                .unwrap_or(false);
            if !has_files {
                anyhow::bail!("Runner `{}` not found.", version);
            }

            Ok(umu_run)
        }
    }
}

fn resolve_steam_runner(version: &str) -> Result<PathBuf> {
    crate::steam::local::find_proton_install(version)
        .ok_or_else(|| anyhow::anyhow!("Runner `{}` not found.", version))?;
    find_umu_run().ok_or_else(|| {
        anyhow::Error::new(ComponentMissing {
            name: "umu-run".to_string(),
        })
    })
}

fn find_executable_in_paths(names: &[&str], extra_paths: &[&str]) -> Option<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            for name in names {
                let full_path = Path::new(dir).join(name);
                if full_path.exists() && is_executable(&full_path) {
                    return Some(full_path);
                }
            }
        }
    }
    for path in extra_paths {
        let expanded = shellexpand::tilde(path);
        let p = Path::new(expanded.as_ref());
        if p.exists() && is_executable(p) {
            return Some(p.to_path_buf());
        }
    }
    None
}

fn find_umu_run() -> Option<PathBuf> {
    const SYSTEM_PATHS: &[&str] = &[
        "/app/share/umu/umu-run",
        "/usr/share/umu/umu-run",
        "/usr/local/share/umu/umu-run",
        "/opt/umu/umu-run",
    ];
    if let Some(p) = find_executable_in_paths(&["umu-run", "umu_run.py"], SYSTEM_PATHS) {
        return Some(p);
    }
    let our_runtime = runtime_dir().join("umu-run");
    (our_runtime.exists() && is_executable(&our_runtime)).then_some(our_runtime)
}

fn find_native_steam() -> Option<String> {
    const STEAM_PATHS: &[&str] = &[
        "~/.steam/steam.sh",
        "~/.steam/steam/steam.sh",
        "~/.local/share/Steam/steam.sh",
    ];
    find_executable_in_paths(&["steam", "steam.sh"], STEAM_PATHS)
        .map(|p| p.to_string_lossy().to_string())
}

fn flatpak_steam_installed() -> bool {
    dirs::home_dir()
        .map(|h| h.join(".var/app/com.valvesoftware.Steam").exists())
        .unwrap_or(false)
}

fn build_steam_command(appid: &str, args: &[String]) -> Vec<String> {
    if std::env::var("FLATPAK_ID").is_ok() {
        let uri = if args.is_empty() {
            format!("steam://rungameid/{}", appid)
        } else {
            format!("steam://run/{}//{}/", appid, args.join(" "))
        };
        return vec!["xdg-open".to_string(), uri];
    }

    if let Some(exe) = find_native_steam() {
        let mut cmd = vec![exe, "-applaunch".to_string(), appid.to_string()];
        cmd.extend(args.iter().cloned());
        return cmd;
    }

    if flatpak_steam_installed() {
        let mut cmd = vec![
            "flatpak".to_string(),
            "run".to_string(),
            "com.valvesoftware.Steam".to_string(),
            "-applaunch".to_string(),
            appid.to_string(),
        ];
        cmd.extend(args.iter().cloned());
        return cmd;
    }

    vec![
        "xdg-open".to_string(),
        format!("steam://rungameid/{}", appid),
    ]
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

pub fn prefix_path_for(game: &Game) -> PathBuf {
    if !game.wine.prefix.is_empty() {
        return PathBuf::from(TemplateVars::base(game).expand(&game.wine.prefix));
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
    dir.join(folder)
}

pub fn effective_prefix(game: &Game) -> Option<PathBuf> {
    match game.runner.runner_type.as_str() {
        "native" | "flatpak" => None,
        "steam" => {
            if game.source.app_id.is_empty() {
                None
            } else {
                crate::steam::local::find_steam_prefix(&game.source.app_id)
            }
        }
        _ => Some(prefix_path_for(game)),
    }
}

pub fn resolve_prefix(game: &Game) -> PathBuf {
    let prefix = prefix_path_for(game);
    if game.wine.prefix.is_empty()
        && !prefix.exists()
        && let Err(e) = std::fs::create_dir_all(&prefix)
    {
        tracing::error!("failed to create prefix dir: {}", e);
    }
    prefix
}

fn resolve_working_dir(game: &Game) -> PathBuf {
    if game.launch.working_dir.is_empty() {
        game.metadata
            .exe
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(&game.launch.working_dir)
    }
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
        assert_eq!(
            WineVariant::from_version("wine-ge-9-5"),
            WineVariant::WineGE
        );
        assert_eq!(WineVariant::from_version("lutris-7.2"), WineVariant::WineGE);
        assert_eq!(
            WineVariant::from_version("GE-Proton10-34"),
            WineVariant::Proton
        );
        assert_eq!(
            WineVariant::from_version("Proton-9-0-4"),
            WineVariant::Proton
        );
    }

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
            WineVariant::WineGE,
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
}
