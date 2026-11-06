use clap::{Parser, Subcommand};
use omikuji_core::app_settings::AppSettings;
use omikuji_core::library::{Game, Library, id_from_slug_id};
use omikuji_core::process::{ErrorAction, ErrorNotification};
use omikuji_core::{desktop, launch, process, updates};

use crate::single_instance;
use std::io::{self, IsTerminal, Write};
use unicode_width::UnicodeWidthStr;

#[derive(Parser)]
#[command(
    name = "omikuji",
    version,
    about = "Qt/QML based wine apps launcher for Linux",
    disable_help_subcommand = true,
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Cmd>,
    #[arg(
        value_name = "FILE",
        help = "Windows executable to run via a wine runner + prefix picker"
    )]
    pub file: Option<String>,
}

#[derive(Subcommand)]
pub enum Cmd {
    #[command(about = "Launch a game by slug, id, or slug_id")]
    Run {
        target: String,
        #[arg(
            long,
            help = "Report failures in the omikuji window instead of on stderr"
        )]
        notify_gui: bool,
    },
    #[command(about = "List the games in the library")]
    List,
    #[command(about = "Open omikuji in console (big picture) mode")]
    Console,
}

pub enum CliAction {
    Exit(i32),
    Gui,
    Console,
    RunExe(String),
}

pub fn dispatch() -> CliAction {
    let cli = Cli::parse();

    match cli.command {
        Some(Cmd::Run { target, notify_gui }) => {
            let handle = std::thread::spawn(move || run_game(&target, notify_gui));
            match handle.join().unwrap_or(1) {
                0 => CliAction::Exit(0),
                _ if notify_gui => CliAction::Gui,
                code => CliAction::Exit(code),
            }
        }
        Some(Cmd::List) => CliAction::Exit(list_games()),
        Some(Cmd::Console) => CliAction::Console,
        None => {
            if let Some(file) = cli.file {
                CliAction::RunExe(file)
            } else if AppSettings::load().console_mode.active {
                CliAction::Console
            } else {
                CliAction::Gui
            }
        }
    }
}

struct Report {
    gui: bool,
    game_id: String,
}

impl Report {
    fn error(&self, title: &str, message: impl Into<String>, action: ErrorAction) {
        let message = message.into();
        tracing::error!("{message}");
        if self.gui {
            process::notify_error(ErrorNotification {
                game_id: self.game_id.clone(),
                title: title.to_string(),
                message,
                action,
            });
        }
    }
}

fn run_game(input: &str, notify_gui: bool) -> i32 {
    let mut report = Report {
        gui: notify_gui,
        game_id: String::new(),
    };

    let library = match Library::load() {
        Ok(l) => l,
        Err(e) => {
            report.error(
                "Couldn't launch",
                format!("Failed to load the library: {e}"),
                ErrorAction::None,
            );
            return 1;
        }
    };

    let game = match resolve_target(&library, input) {
        Resolved::Found(g) => g.clone(),
        Resolved::Multiple(matches) => match pick_from_matches(&matches) {
            Some(idx) => matches[idx].clone(),
            None => {
                report.error(
                    "Couldn't launch",
                    format!("Several games match `{input}`. Launch it by its id."),
                    ErrorAction::None,
                );
                return 2;
            }
        },
        Resolved::NotFound => {
            report.error(
                "Couldn't launch",
                format!("No game matches `{input}`"),
                ErrorAction::None,
            );
            return 1;
        }
    };

    report.game_id = game.metadata.id.clone();
    launch_and_wait(&game, &report)
}

enum Resolved<'a> {
    Found(&'a Game),
    Multiple(Vec<&'a Game>),
    NotFound,
}

fn resolve_target<'a>(lib: &'a Library, input: &str) -> Resolved<'a> {
    let lower = input.to_lowercase();
    let id = id_from_slug_id(&lower);

    if let Some(g) = lib
        .game
        .iter()
        .find(|g| g.metadata.id.eq_ignore_ascii_case(id))
    {
        return Resolved::Found(g);
    }

    let matches: Vec<&Game> = lib
        .game
        .iter()
        .filter(|g| desktop::game_slug(g).eq_ignore_ascii_case(&lower))
        .collect();

    match matches.len() {
        0 => Resolved::NotFound,
        1 => Resolved::Found(matches[0]),
        _ => Resolved::Multiple(matches),
    }
}

fn pick_from_matches(matches: &[&Game]) -> Option<usize> {
    if !io::stdin().is_terminal() {
        eprintln!("multiple games match - re-run with the id for precision:");
        print_matches(matches);
        return None;
    }

    eprintln!("multiple games match:");
    print_matches(matches);
    eprint!("Pick [1-{}]: ", matches.len());
    let _ = io::stderr().flush();

    let mut buf = String::new();
    if io::stdin().read_line(&mut buf).is_err() {
        return None;
    }
    let choice: usize = buf.trim().parse().ok()?;
    if choice == 0 || choice > matches.len() {
        return None;
    }
    Some(choice - 1)
}

fn print_matches(matches: &[&Game]) {
    for (i, g) in matches.iter().enumerate() {
        let last = if g.metadata.last_played.is_empty() {
            "never"
        } else {
            &g.metadata.last_played
        };
        eprintln!(
            "  {}) {}  -  {}  -  {}  -  {}",
            i + 1,
            g.metadata.name,
            g.metadata.id,
            playtime_hours(g),
            last
        );
    }
}

fn playtime_hours(game: &Game) -> String {
    format!("{:.1}h", game.metadata.playtime)
}

fn list_games() -> i32 {
    let library = match Library::load() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to load the library: {e}");
            return 1;
        }
    };

    let mut games: Vec<&Game> = library.game.iter().collect();
    games.sort_by_key(|g| g.metadata.name.to_lowercase());

    let rows: Vec<[String; 4]> = games
        .iter()
        .map(|g| {
            [
                g.metadata.name.clone(),
                g.metadata.id.clone(),
                playtime_hours(g),
                g.runner.runner_type.as_str().to_owned(),
            ]
        })
        .collect();

    let header = ["Name", "ID", "Playtime", "Type"].map(String::from);
    let widths: [usize; 4] = std::array::from_fn(|column| {
        rows.iter()
            .chain([&header])
            .map(|row| row[column].width())
            .max()
            .unwrap_or_default()
    });

    let mut out = io::stdout().lock();
    for row in [&header].into_iter().chain(&rows) {
        let line = row
            .iter()
            .zip(widths)
            .map(|(cell, width)| format!("{cell}{}", " ".repeat(width - cell.width())))
            .collect::<Vec<_>>()
            .join("  ");
        let _ = writeln!(out, "{}", line.trim_end());
    }

    0
}

fn launch_and_wait(game: &Game, report: &Report) -> i32 {
    if report.gui && single_instance::hand_off_launch(&game.metadata.id) {
        return 0;
    }

    let Some(_guard) = process::try_claim_launch(&game.metadata.id) else {
        report.error(
            "Already running",
            "This game is already running or still starting up.",
            ErrorAction::None,
        );
        return 1;
    };

    if report.gui
        && let Some(info) = updates::pre_launch_check(game)
    {
        process::notify_update_required(info);
        return 1;
    }

    let config = match launch::prepare_launch(game) {
        Ok(c) => c,
        Err(e) => {
            report.error(
                "Couldn't launch",
                e.to_string(),
                ErrorAction::for_launch_error(&e),
            );
            return 1;
        }
    };

    tracing::info!("launching '{}'", game.metadata.name);

    let game_id = game.metadata.id.clone();
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            report.error(
                "Couldn't launch",
                format!("Failed to start the tokio runtime: {e}"),
                ErrorAction::None,
            );
            return 1;
        }
    };

    if let Err(e) = rt.block_on(process::launch_game(&config)) {
        report.error(
            "Couldn't launch",
            e.to_string(),
            ErrorAction::for_launch_error(&e),
        );
        return 1;
    }

    let stop_id = game_id.clone();
    process::on_stop_signal(move || {
        process::stop_game(&stop_id);
    });

    while process::is_game_running(&game_id) {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    0
}
