use cxx_qt_lib::QString;

impl super::qobject::GameModel {
    pub fn changelog_pending(&self) -> QString {
        let current = env!("CARGO_PKG_VERSION");
        let last_seen = omikuji_core::app_settings::AppSettings::load()
            .state
            .last_seen_changelog;
        if last_seen.is_empty() {
            return QString::default();
        }
        match omikuji_core::changelog::notes_since(&last_seen, current) {
            Some(body) => QString::from(
                &serde_json::json!({ "from": last_seen, "to": current, "body": body }).to_string(),
            ),
            None => QString::default(),
        }
    }

    pub fn mark_changelog_seen(&self) {
        let mut settings = omikuji_core::app_settings::AppSettings::load();
        settings.state.last_seen_changelog = env!("CARGO_PKG_VERSION").to_string();
        if let Err(e) = settings.save() {
            tracing::error!("failed to persist last_seen_changelog: {}", e);
        }
    }
}
