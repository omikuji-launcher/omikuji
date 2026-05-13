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

    pub fn drain_file_dialog_results(mut self: Pin<&mut Self>) {
        for r in omikuji_core::install_sizes::take_file_dialog_pending() {
            self.as_mut().file_dialog_result(
                &QString::from(&r.request_id),
                &QString::from(&r.path),
            );
        }
    }

    pub fn drain_install_sizes(mut self: Pin<&mut Self>) {
        for r in omikuji_core::install_sizes::take_pending() {
            let payload = serde_json::json!({
                "download": r.download_bytes.to_string(),
                "install": r.install_bytes.to_string(),
                "error": r.error,
            })
            .to_string();
            self.as_mut().install_size_result(
                &QString::from(&r.request_id),
                &QString::from(&payload),
            );
        }
    }
}
