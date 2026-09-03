use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::ComponentMissing;
use super::env::{EnvPurpose, build_env, game_env_pairs};
use super::prefix::resolve_prefix;
use super::wine::{WineVariant, resolve_wine_exe};
use crate::library::Game;
use crate::store::steam::local::{find_native_steam, flatpak_steam_installed};
use crate::template_vars::TemplateVars;

pub struct ResolvedLaunch {
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: PathBuf,
    pub game_id: String,
    pub game_name: String,
    pub post_exit_script: String,
}

impl ResolvedLaunch {
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

    pub fn to_command(&self) -> Result<Command> {
        let (program, args) = self
            .command
            .split_first()
            .ok_or_else(|| anyhow::anyhow!("no launch command for {}", self.game_name))?;
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd.current_dir(&self.working_dir);
        cmd.env_clear();
        cmd.envs(&self.env);
        Ok(cmd)
    }
}

pub(super) fn assemble_launch(game: &Game) -> Result<ResolvedLaunch> {
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
        let legendary = crate::store::epic::source::find_legendary().ok_or_else(|| {
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
            app_id.clone(),
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
            cmd.push(game.metadata.exe.to_string_lossy().to_string());
        }
        for arg in &game.launch.args {
            cmd.push(arg.clone());
        }
        cmd
    };

    apply_wrapping(&mut command, &mut env, game, true);

    Ok(ResolvedLaunch::from_game(game, command, env, working_dir))
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

fn build_steam_launch(game: &Game, working_dir: PathBuf) -> Result<ResolvedLaunch> {
    let appid = effective_app_id(game);
    if appid.is_empty() {
        anyhow::bail!("Steam runner requires an Application ID");
    }

    let mut command = build_steam_command(&appid, &game.launch.args);

    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.extend(game_env_pairs(game));

    apply_wrapping(&mut command, &mut env, game, true);

    Ok(ResolvedLaunch::from_game(game, command, env, working_dir))
}

fn build_flatpak_launch(game: &Game, working_dir: PathBuf) -> Result<ResolvedLaunch> {
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

    Ok(ResolvedLaunch::from_game(game, command, env, working_dir))
}

fn build_native_launch(game: &Game, working_dir: PathBuf) -> Result<ResolvedLaunch> {
    let exe = &game.metadata.exe;
    let mut command = vec![relative_exe(exe, &working_dir)];
    for arg in &game.launch.args {
        command.push(arg.clone());
    }

    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.extend(game_env_pairs(game));

    apply_wrapping(&mut command, &mut env, game, true);

    Ok(ResolvedLaunch::from_game(game, command, env, working_dir))
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
