use clap::{Parser, Subcommand};
use omikuji_core::library::{Game, Library};
use omikuji_core::ui_settings::UiSettings;
use omikuji_core::{desktop, launch, process};
use std::io::{self, IsTerminal, Write};

#[derive(Parser)]
#[command(
    name = "omikuji",
    version,
    about = "Qt/QML based wine apps launcher for Linux",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Cmd>,
}

#[derive(Subcommand)]
pub enum Cmd {
    #[command(about = "Launch a game by slug, id, or slug_id")]
    Run { target: String },
    #[command(about = "Open omikuji in console (big picture) mode")]
    Console,
}

pub enum CliAction {
    Exit(i32),
    Gui,
    Console,
}

pub fn dispatch() -> CliAction {
    let cli = Cli::parse();

    match cli.command {
        Some(Cmd::Run { target }) => {
            let handle = std::thread::spawn(move || run_game(&target));
            CliAction::Exit(handle.join().unwrap_or(1))
        }
        Some(Cmd::Console) => CliAction::Console,
        None => {
            if UiSettings::load().console_mode.active {
                CliAction::Console
            } else {
                CliAction::Gui
            }
        }
    }
}

fn run_game(input: &str) -> i32 {
    let library = match Library::load() {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("failed to load library: {}", e);
            return 1;
        }
    };

    let game = match resolve_target(&library, input) {
        Resolved::Found(g) => g.clone(),
        Resolved::Multiple(matches) => match pick_from_matches(&matches) {
            Some(idx) => matches[idx].clone(),
            None => return 2,
        },
        Resolved::NotFound => {
            tracing::error!("no game matches '{}'", input);
            return 1;
        }
    };

    launch_and_wait(&game)
}

enum Resolved<'a> {
    Found(&'a Game),
    Multiple(Vec<&'a Game>),
    NotFound,
}

fn resolve_target<'a>(lib: &'a Library, input: &str) -> Resolved<'a> {
    let lower = input.to_lowercase();

    if let Some(g) = lib
        .game
        .iter()
        .find(|g| desktop::launch_target(g).eq_ignore_ascii_case(&lower))
    {
        return Resolved::Found(g);
    }

    if let Some(g) = lib
        .game
        .iter()
        .find(|g| g.metadata.id.eq_ignore_ascii_case(&lower))
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
        eprintln!("multiple games match - re-run with slug_id for precision:");
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
            "  {}) {}  -  {}  -  {:.1}h  -  {}",
            i + 1,
            g.metadata.name,
            g.metadata.id,
            g.metadata.playtime,
            last
        );
    }
}

fn launch_and_wait(game: &Game) -> i32 {
    if process::is_game_running(&game.metadata.id) {
        tracing::warn!("'{}' is already running", game.metadata.name);
        return 1;
    }

    let config = match launch::build_launch(game) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to build launch config: {}", e);
            return 1;
        }
    };

    tracing::info!("launching '{}'", game.metadata.name);

    let game_id = game.metadata.id.clone();
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("failed to start tokio runtime: {}", e);
            return 1;
        }
    };

    if let Err(e) = rt.block_on(process::launch_game(&config)) {
        tracing::error!("failed to launch: {}", e);
        return 1;
    }

    while process::is_game_running(&game_id) {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    0
}
