use std::borrow::Cow;
use std::path::PathBuf;

use crate::library::{Game, SourceKind};
use crate::store::{epic, gog};

const PRIMARY_ID: &str = "primary";

#[derive(Debug, Clone, PartialEq)]
pub struct LaunchOption {
    pub id: String,
    pub name: String,
    pub exe: Option<PathBuf>,
    pub working_dir: Option<PathBuf>,
    pub args: Vec<String>,
}

impl LaunchOption {
    pub fn with_args(name: String, args: Vec<String>) -> Self {
        Self {
            id: name.clone(),
            name,
            exe: None,
            working_dir: None,
            args,
        }
    }

    fn primary(name: &str) -> Self {
        Self {
            id: PRIMARY_ID.to_string(),
            ..Self::with_args(name.to_string(), Vec::new())
        }
    }

    fn apply_to(&self, game: &mut Game) {
        match &self.exe {
            Some(exe) => {
                game.metadata.exe = exe.clone();
                game.launch.args = self.args.clone();
            }
            None => {
                game.launch.args.splice(0..0, self.args.iter().cloned());
            }
        }
        if let Some(dir) = &self.working_dir {
            game.launch.working_dir = dir.to_string_lossy().into_owned();
        }
    }
}

pub fn list(game: &Game) -> Vec<LaunchOption> {
    let app_id = game.effective_app_id();
    let extras = match game.source.kind {
        SourceKind::Epic => epic::launch_options(app_id),
        SourceKind::Gog => gog::launch_options(app_id),
        _ => Vec::new(),
    };
    if extras.is_empty() {
        return extras;
    }
    std::iter::once(LaunchOption::primary(&game.metadata.name))
        .chain(extras)
        .collect()
}

pub fn choice_missing(game: &Game) -> bool {
    let options = list(game);
    !options.is_empty() && !options.iter().any(|o| o.id == game.launch.store_option)
}

pub fn find(game: &Game, id: &str) -> Option<LaunchOption> {
    list(game).into_iter().find(|o| o.id == id)
}

pub fn applied(game: &Game) -> Cow<'_, Game> {
    match find(game, &game.launch.store_option) {
        Some(option) if option.id != PRIMARY_ID => {
            let mut game = game.clone();
            option.apply_to(&mut game);
            Cow::Owned(game)
        }
        _ => Cow::Borrowed(game),
    }
}
