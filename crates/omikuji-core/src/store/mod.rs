pub mod cache;
pub mod epic;
pub mod game;
pub mod gog;
pub mod nile;
pub mod registry;
pub mod steam;

use crate::downloads;
use crate::library::{Game, Library, SourceKind};
use anyhow::Result;

pub use game::StoreGame;

pub struct UpdateInfo {
    pub from_version: String,
    pub to_version: String,
}

pub fn uninstall(game: &Game) -> Result<()> {
    let app_id = &game.source.app_id;
    if app_id.is_empty() {
        anyhow::bail!("'{}' has no store app id", game.metadata.name);
    }
    downloads::manager().cancel_app(app_id);
    match game.source.kind {
        SourceKind::Epic => epic::uninstall(app_id)?,
        SourceKind::Gog => gog::uninstall(app_id, &game.metadata.name)?,
        SourceKind::Nile => nile::uninstall(app_id)?,
        kind => anyhow::bail!("{} games have no store uninstall", kind.as_str()),
    }
    if let Err(e) = Library::remove_game_file(&game.metadata.id) {
        tracing::error!("failed to remove game file: {}", e);
    }
    Ok(())
}
