use cxx_qt_lib::QString;
use std::pin::Pin;

impl super::qobject::GameModel {
    pub fn drain_notifications(mut self: Pin<&mut Self>) {
        for n in omikuji_core::notifications::take_pending() {
            self.as_mut().notification(
                &QString::from(n.level.as_str()),
                &QString::from(&n.title),
                &QString::from(&n.message),
            );
        }
    }

    pub fn drain_launch_requests(mut self: Pin<&mut Self>) {
        for request in omikuji_core::process::take_launch_requests() {
            let index = self
                .library
                .game
                .iter()
                .position(|g| g.metadata.id == request.game_id)
                .map(|i| i as i32);

            match index {
                Some(i) if self.as_mut().launch_game(i) => continue,
                Some(_) => {}
                None => {
                    omikuji_core::process::notify_error(omikuji_core::process::ErrorNotification {
                        game_id: request.game_id.clone(),
                        title: "Couldn't launch".to_string(),
                        message: "This game is no longer in the library.".to_string(),
                        action: omikuji_core::process::ErrorAction::None,
                    })
                }
            }
            request.release();
        }
    }

    pub fn drain_update_notifications(mut self: Pin<&mut Self>) {
        for n in omikuji_core::process::take_update_notifications() {
            let display_name = self
                .library
                .game
                .iter()
                .find(|g| g.metadata.id == n.game_id)
                .map(|g| g.metadata.name.clone())
                .unwrap_or_default();
            self.as_mut().update_required(
                &QString::from(&n.game_id),
                &QString::from(&n.app_id),
                &QString::from(&display_name),
                &QString::from(&n.from_version),
                &QString::from(&n.to_version),
                &QString::from(&n.download_size.to_string()),
                n.can_diff,
                n.delta_supported,
            );
        }
    }

    pub fn drain_errors(mut self: Pin<&mut Self>) {
        for n in omikuji_core::process::take_errors() {
            let display_name = self
                .library
                .game
                .iter()
                .find(|g| g.metadata.id == n.game_id)
                .map(|g| g.metadata.name.clone())
                .unwrap_or_default();
            self.as_mut().error_required(
                &QString::from(&n.game_id),
                &QString::from(&display_name),
                &QString::from(&n.title),
                &QString::from(&n.message),
                &QString::from(n.action.as_str()),
            );
        }
    }

    pub fn drain_install_sizes(mut self: Pin<&mut Self>) {
        for r in omikuji_core::install_sizes::take_pending() {
            let payload = serde_json::json!({
                "download": r.download_bytes.to_string(),
                "install": r.install_bytes.to_string(),
                "launchExe": r.launch_exe,
                "error": r.error,
            })
            .to_string();
            self.as_mut()
                .install_size_result(&QString::from(&r.request_id), &QString::from(&payload));
        }
    }

    pub fn drain_game_details(mut self: Pin<&mut Self>) {
        for r in omikuji_core::install_sizes::take_details_pending() {
            self.as_mut()
                .game_details_result(&QString::from(&r.request_id), &QString::from(&r.payload));
        }
    }
}
