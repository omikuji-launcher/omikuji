use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use omikuji_core::library::{Game, Library};
use omikuji_core::media;

impl super::qobject::GameModel {
    pub fn register_game_json(mut self: Pin<&mut Self>, game_json: &QString) -> QString {
        let mut game: Game = match serde_json::from_str(&game_json.to_string()) {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("register_game_json: invalid game json: {e}");
                return QString::default();
            }
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        if let Err(e) = Library::save_game_static(&game) {
            tracing::error!("register_game_json: failed to save: {e}");
            return QString::default();
        }

        let id = game.metadata.id.clone();
        let name_for_media = game.metadata.name.clone();
        let qt_thread = self.as_mut().qt_thread();
        let on_asset = super::media_changed_notifier(qt_thread, id.clone());
        let id_for_media = id.clone();
        std::thread::spawn(move || {
            media::fetch_media_blocking_with(
                media::MediaSlot::Live,
                &id_for_media,
                &name_for_media,
                on_asset,
            );
        });

        self.as_mut().insert_game_sorted(game);
        QString::from(&id)
    }
}
