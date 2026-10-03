use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;

use fs_err::OpenOptions;

use crate::game_logs;
use crate::launch::{self, ResolvedLaunch};
use crate::library::Game;
use crate::runners;

pub(super) fn spawn_writer(game_id: String, log_path: Option<PathBuf>) -> Sender<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || {
        let mut file = log_path
            .as_ref()
            .and_then(|p| OpenOptions::new().create(true).append(true).open(p).ok());
        while let Ok(line) = rx.recv() {
            if let Some(ref mut f) = file {
                let _ = writeln!(f, "{}", line);
            }
            game_logs::append_line(&game_id, line);
        }
    });
    tx
}

// yes pump as in the sexual joke ghaha yeah mature of me
pub(super) fn pump_lines(pipe: impl Read + Send + 'static, tx: Sender<String>) {
    thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(|l| l.ok()) {
            let _ = tx.send(line);
        }
    });
}

pub(super) fn header(game: &Game, config: &ResolvedLaunch) -> Vec<String> {
    let section = |title: &str| format!("=== {} ===", title.to_uppercase());
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    let mut lines = vec![
        section(&format!("omikuji {} - {now}", env!("CARGO_PKG_VERSION"))),
        String::new(),
        section("environment"),
    ];
    lines.extend(
        launch::env_overrides(&config.env)
            .into_iter()
            .map(|(k, v)| format!("{k}={v}")),
    );

    lines.extend([
        String::new(),
        section("game"),
        format!("name: {}", game.metadata.name),
        format!("library_id: {}", config.game_id),
        format!("runner: {}", runner_label(game)),
    ]);
    if !game.metadata.exe.as_os_str().is_empty() {
        lines.push(format!("exe: {}", game.metadata.exe.display()));
    }
    if !game.launch.args.is_empty() {
        lines.push(format!("args: {}", game.launch.args.join(" ")));
    }
    lines.extend([
        format!("working_dir: {}", config.working_dir.display()),
        format!("command: {}", config.command.join(" ")),
        String::new(),
        section("output"),
    ]);
    lines
}

fn runner_label(game: &Game) -> String {
    let kind = game.runner.runner_type.as_str();
    if !game.uses_wine_prefix() {
        return kind.to_string();
    }
    let version = &game.wine.version;
    match runners::latest_source(version).and_then(|s| runners::latest_installed_tag(&s)) {
        Some(tag) => format!("{kind} - {version} ({tag})"),
        None => format!("{kind} - {version}"),
    }
}
