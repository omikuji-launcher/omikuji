use std::pin::Pin;

use cxx_qt_lib::QString;

use omikuji_core::app_settings::AppSettings;
use omikuji_core::components_config::ArchiveSource;
use omikuji_core::library::Game;
use omikuji_core::process::{self, ErrorAction};
use omikuji_core::template_vars::TemplateVars;
use omikuji_core::wine_tools::{self, WineTool};
use omikuji_core::{anyhow, game_logs, launch, log_files, notifications, runners, updates};

use crate::inhibit;

impl super::qobject::GameModel {
    pub fn launch_game(mut self: Pin<&mut Self>, index: i32) -> bool {
        use cxx_qt::Threading;
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx).cloned() else {
            tracing::warn!("launch_game: invalid index {}", index);
            return false;
        };

        if refuse_if_busy(&game) {
            return false;
        }

        process::mark_launching(&game.metadata.id);

        let qt = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            if let Some(info) = updates::pre_launch_check(&game) {
                process::clear_launching(&game.metadata.id);
                process::release_exit_waiters(&game.metadata.id);
                process::notify_update_required(info);
                return;
            }
            if do_spawn_launch(&game) {
                let gid = game.metadata.id.clone();
                let _ = qt.queue(move |mut obj: Pin<&mut super::qobject::GameModel>| {
                    obj.as_mut().launch_proceeding(&QString::from(&gid));
                });
            }
        });
        true
    }

    pub fn launch_game_force(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            tracing::warn!("launch_game_force: invalid index {}", index);
            return false;
        };
        self.try_spawn_launch(game)
    }

    pub fn launch_exe(&self, exe: &QString, runner: &QString, prefix: &QString) -> bool {
        let exe_path = std::path::PathBuf::from(exe.to_string());
        let name = exe_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Program")
            .to_string();
        let game = Game::new(name, exe_path)
            .with_prefix(prefix.to_string())
            .with_runner_version(runner.to_string());
        let config = match launch::build_launch(&game) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("run-exe build_launch failed: {}", e);
                return false;
            }
        };
        if config.command.is_empty() {
            return false;
        }
        let mut cmd = std::process::Command::new(&config.command[0]);
        cmd.args(&config.command[1..]);
        cmd.current_dir(&config.working_dir);
        cmd.env_clear();
        cmd.envs(&config.env);
        cmd.stdin(std::process::Stdio::null());
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        match cmd.spawn() {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("run-exe spawn failed: {}", e);
                false
            }
        }
    }

    pub fn run_exe_path(&self) -> QString {
        QString::from(&std::env::var("OMIKUJI_RUN_EXE").unwrap_or_default())
    }

    pub fn quit_now(&self) {
        unsafe { libc::_exit(0) }
    }

    fn try_spawn_launch(&self, game: &Game) -> bool {
        if refuse_if_busy(game) {
            return false;
        }

        process::mark_launching(&game.metadata.id);
        do_spawn_launch(game)
    }

    pub fn stop_game(&self, game_id: &QString) {
        let id = game_id.to_string();
        tracing::info!("requesting stop for game '{}'", id);
        process::stop_game(&id);
    }

    pub fn run_wine_tool(&self, game_id: &QString, tool: &QString) {
        let id = game_id.to_string();
        let tool_name = tool.to_string();
        let Some(game) = self.library.game(&id).cloned() else {
            tracing::warn!("game '{}' not found", id);
            return;
        };
        let Some(t) = WineTool::from_name(&tool_name) else {
            tracing::warn!("unknown tool '{}'", tool_name);
            return;
        };
        let display_name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let tool_label = tool_name.clone();

        let Some(guard) = wine_tools::try_start(&id, &tool_name) else {
            notifications::warning(&display_name, format!("{} is already starting", tool_label));
            return;
        };

        // prefix-init and umu-run startup can be slow, detach so the ui doesnt block
        std::thread::spawn(move || {
            let _guard = guard;
            match wine_tools::run(&game, t) {
                Ok(_child) => {
                    notifications::info(&display_name, format!("Opened {}", tool_label));
                }
                Err(e) => {
                    process::notify_error(process::ErrorNotification {
                        game_id: game_id_owned,
                        title: format!("{} failed", tool_label),
                        message: format!("{}", e),
                        action: ErrorAction::OpenGameSettings,
                    });
                }
            }
        });
    }

    pub fn run_wine_command(mut self: Pin<&mut Self>, game_id: &QString, command: &QString) {
        use cxx_qt::Threading;
        if self.wine_command_running {
            return;
        }
        let id = game_id.to_string();
        let Some(game) = self.library.game(&id).cloned() else {
            tracing::warn!("game '{}' not found", id);
            return;
        };
        let Some(tool) = WineTool::from_command_line(&command.to_string()) else {
            return;
        };
        self.as_mut().set_wine_command_running(true);
        let qt = self.as_mut().qt_thread();
        let line_qt = qt.clone();
        wine_tools::run_detached(
            game,
            tool,
            move |line| {
                let l = line.to_string();
                let _ = line_qt.queue(move |mut obj: Pin<&mut super::qobject::GameModel>| {
                    obj.as_mut().wine_command_output(&QString::from(&l));
                });
            },
            move |ok, err| {
                let _ = qt.queue(move |mut obj: Pin<&mut super::qobject::GameModel>| {
                    obj.as_mut().set_wine_command_running(false);
                    obj.as_mut().wine_command_finished(ok, &QString::from(&err));
                });
            },
        );
    }

    pub fn expand_vars(&self, text: &QString) -> QString {
        let vars = match self.draft.as_ref() {
            Some(draft) => TemplateVars::for_game(&draft.game),
            None => TemplateVars::global(),
        };
        QString::from(&vars.expand(&text.to_string()))
    }

    pub fn expand_global_vars(&self, text: &QString) -> QString {
        QString::from(&TemplateVars::global().expand(&text.to_string()))
    }

    pub fn expand_game_vars(&self, game_id: &QString, text: &QString) -> QString {
        let id = game_id.to_string();
        match self.library.game(&id) {
            Some(game) => QString::from(&TemplateVars::for_game(game).expand(&text.to_string())),
            None => text.clone(),
        }
    }

    pub fn run_wine_exe(&self, game_id: &QString, exe_path: &QString) {
        let id = game_id.to_string();
        let exe = exe_path.to_string();
        if exe.is_empty() {
            return;
        }
        let Some(game) = self.library.game(&id).cloned() else {
            tracing::warn!("game '{}' not found", id);
            return;
        };
        let display_name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let path = std::path::PathBuf::from(&exe);
        let file_label = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| exe.clone());
        std::thread::spawn(
            move || match wine_tools::run(&game, WineTool::RunExe(path)) {
                Ok(_child) => {
                    notifications::info(&display_name, format!("Running {}", file_label));
                }
                Err(e) => {
                    process::notify_error(process::ErrorNotification {
                        game_id: game_id_owned,
                        title: "Couldn't run executable".to_string(),
                        message: format!("`{}` failed: {}", file_label, e),
                        action: ErrorAction::OpenGameSettings,
                    });
                }
            },
        );
    }

    pub fn check_exited_games(mut self: Pin<&mut Self>) {
        for game_id in process::take_exited_games() {
            inhibit::release(&game_id);
            self.as_mut().game_stopped(&QString::from(&game_id));
        }
    }

    pub fn drain_game_log_events(mut self: Pin<&mut Self>) {
        for id in game_logs::drain_dirty() {
            self.as_mut().game_log_appended(&QString::from(&id));
        }
    }

    pub fn game_log(&self, game_id: &QString) -> QString {
        QString::from(&game_logs::get_log(&game_id.to_string()))
    }

    pub fn clear_game_log(&self, game_id: &QString) {
        game_logs::clear_log(&game_id.to_string());
    }

    pub fn save_game_log(&self, game_id: &QString) -> QString {
        let id = game_id.to_string();
        let body = game_logs::get_log(&id);
        if body.is_empty() {
            return QString::from("");
        }
        let Some(game) = self.library.game(&id) else {
            return QString::from("");
        };
        let file = match log_files::next_log_path(&log_files::game_logs_dir(game), &game.slug()) {
            Ok(file) => file,
            Err(e) => {
                tracing::error!("{e}");
                return QString::from("");
            }
        };
        match fs_err::write(&file, body) {
            Ok(_) => QString::from(file.to_string_lossy().as_ref()),
            Err(e) => {
                tracing::error!("{e}");
                QString::from("")
            }
        }
    }

    pub fn launch_console_mode(&self) {
        AppSettings::set_console_mode_active(true);
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe)
                .arg("console")
                .env("OMIKUJI_BYPASS_SINGLE_INSTANCE", "1")
                .spawn();
        }
        std::process::exit(0);
    }

    pub fn launch_desktop_mode(&self) {
        AppSettings::set_console_mode_active(false);
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe)
                .env("OMIKUJI_BYPASS_SINGLE_INSTANCE", "1")
                .spawn();
        }
        std::process::exit(0);
    }
}

fn pending_latest_runner(game: &Game) -> Option<ArchiveSource> {
    if !game.uses_wine_prefix() {
        return None;
    }
    let source = runners::latest_source(&game.wine.version)?;
    (!runners::latest_dir(&source).exists()).then_some(source)
}

fn blocking_ensure_latest(source: &ArchiveSource) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(runners::ensure_latest(source))?;
    Ok(())
}

fn do_spawn_launch(game: &Game) -> bool {
    if let Err(e) = launch::precheck_exe(game) {
        notify_launch_failed(game.metadata.id.clone(), &e);
        return false;
    }

    if runners::is_latest_updating(&game.wine.version) {
        notify_launch_failed(
            game.metadata.id.clone(),
            &anyhow::anyhow!(
                "{} is being updated right now, try again once it finishes",
                game.wine.version
            ),
        );
        return false;
    }

    if let Some(source) = pending_latest_runner(game) {
        let game = game.clone();
        std::thread::spawn(move || match blocking_ensure_latest(&source) {
            Ok(()) => {
                do_spawn_launch(&game);
            }
            Err(e) => {
                tracing::error!("failed to install {}: {}", source.name, e);
                notify_launch_failed(game.metadata.id.clone(), &e);
            }
        });
        return true;
    }

    if game.runner.runner_type.is_steam() {
        notifications::info(
            &game.metadata.name,
            "Launching through Steam... any errors will show in Steam itself",
        );
    }

    if game.system.prevent_sleep {
        inhibit::acquire(
            &game.metadata.id,
            &format!("Playing {}", game.metadata.name),
        );
    }

    if game.launch.pre_launch_script.is_empty() {
        match launch::build_launch(game) {
            Ok(config) => {
                spawn_launch_thread(config);
                true
            }
            Err(e) => {
                tracing::error!("failed to build launch config: {}", e);
                notify_launch_failed(game.metadata.id.clone(), &e);
                false
            }
        }
    } else {
        let game = game.clone();
        std::thread::spawn(move || match launch::prepare_launch(&game) {
            Ok(config) => spawn_launch_thread(config),
            Err(e) => {
                tracing::error!("failed to build launch config: {}", e);
                notify_launch_failed(game.metadata.id.clone(), &e);
            }
        });
        true
    }
}

fn spawn_launch_thread(config: launch::ResolvedLaunch) {
    tracing::info!("launching '{}': {:?}", config.game_name, config.command);
    let logs_dir = omikuji_core::logs_dir();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            match process::launch_game(&config).await {
                Ok(proc_id) => {
                    process::clear_launching(&config.game_id);
                    tracing::info!(
                        "game '{}' launched, process id: {:?}",
                        config.game_name,
                        proc_id
                    );
                    tracing::debug!("logs: {}", logs_dir.display());
                }
                Err(e) => {
                    tracing::error!("failed to launch '{}': {}", config.game_name, e);
                    process::notify_game_exited(&config.game_id);
                    process::notify_error(process::ErrorNotification {
                        game_id: config.game_id.clone(),
                        title: "Couldn't launch".to_string(),
                        message: e.to_string(),
                        action: ErrorAction::OpenGameSettings,
                    });
                }
            }
        });
    });
}

fn refuse_if_busy(game: &Game) -> bool {
    let id = &game.metadata.id;
    if !process::is_launching(id) && !process::launch_blocked(id) {
        return false;
    }

    tracing::warn!(
        "game '{}' is already running or launching",
        game.metadata.name
    );
    process::notify_error(process::ErrorNotification {
        game_id: id.clone(),
        title: "Couldn't launch".to_string(),
        message: "This game is already running or still starting up.".to_string(),
        action: ErrorAction::None,
    });
    true
}

fn notify_launch_failed(game_id: String, e: &anyhow::Error) {
    process::notify_game_exited(&game_id);
    let action = ErrorAction::for_launch_error(e);
    process::notify_error(process::ErrorNotification {
        game_id,
        title: "Couldn't launch".to_string(),
        message: e.to_string(),
        action,
    });
}
