#![allow(clippy::too_many_arguments)]

use std::path::PathBuf;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QModelIndex, QString, QVariant};

use omikuji_core::downloads::{
    self, DownloadEntry, DownloadEvent, DownloadKind, DownloadRequest, DownloadStatus,
};

include!(concat!(env!("OUT_DIR"), "/download_model_bridge.rs"));

fn status_label(s: &DownloadStatus) -> &'static str {
    s.short()
}

fn error_text(s: &DownloadStatus) -> String {
    if let DownloadStatus::Failed(e) = s {
        e.clone()
    } else {
        String::new()
    }
}

fn kind_label(k: &DownloadKind) -> &'static str {
    match k {
        DownloadKind::Install => "install",
        DownloadKind::Update { .. } => "update",
        DownloadKind::Repair => "repair",
        DownloadKind::ImportExisting => "import",
    }
}

fn role_banner(e: &DownloadEntry) -> QVariant {
    QVariant::from(&QString::from(e.banner_url.as_deref().unwrap_or("")))
}

fn role_status(e: &DownloadEntry) -> QVariant {
    QVariant::from(&QString::from(status_label(&e.status)))
}

fn role_speed(e: &DownloadEntry) -> QVariant {
    QVariant::from(&(e.speed_bps as f64))
}

fn role_bytes_downloaded(e: &DownloadEntry) -> QVariant {
    QVariant::from(&(e.bytes_downloaded as f64))
}

fn role_bytes_total(e: &DownloadEntry) -> QVariant {
    QVariant::from(&(e.bytes_total as f64))
}

fn role_error(e: &DownloadEntry) -> QVariant {
    QVariant::from(&QString::from(&error_text(&e.status)))
}

fn role_kind(e: &DownloadEntry) -> QVariant {
    QVariant::from(&QString::from(kind_label(&e.kind)))
}

impl Default for DownloadModelRust {
    fn default() -> Self {
        let entries = downloads::manager().list();
        let c = recompute(&entries, "");
        Self {
            count: entries.len() as i32,
            active_count: c.active,
            completed_count: c.completed,
            running_count: c.running,
            queued_count: c.queued,
            failed_count: c.failed,
            hero_id: QString::from(&c.hero_id),
            entries,
        }
    }
}

#[derive(Default)]
struct Counts {
    active: i32,
    completed: i32,
    running: i32,
    queued: i32,
    failed: i32,
    hero_id: String,
}

fn recompute(entries: &[DownloadEntry], prev_hero: &str) -> Counts {
    let mut c = Counts {
        hero_id: entries
            .iter()
            .find(|e| e.status.is_running())
            .or_else(|| {
                entries
                    .iter()
                    .find(|e| e.id == prev_hero && e.status == DownloadStatus::Paused)
            })
            .or_else(|| entries.iter().find(|e| e.status == DownloadStatus::Paused))
            .map(|e| e.id.clone())
            .unwrap_or_default(),
        ..Counts::default()
    };
    for e in entries {
        if e.status.is_active() {
            c.active += 1;
        }
        if e.status.is_running() {
            c.running += 1;
        }
        match &e.status {
            DownloadStatus::Completed => c.completed += 1,
            DownloadStatus::Failed(_) => c.failed += 1,
            DownloadStatus::Queued | DownloadStatus::Paused if e.id != c.hero_id => c.queued += 1,
            _ => {}
        }
    }
    c
}

impl qobject::DownloadModel {
    fn enqueue_epic(
        self: Pin<&mut Self>,
        app_id: &QString,
        display_name: &QString,
        banner_url: &QString,
        install_path: &QString,
        prefix_path: &QString,
        runner_version: &QString,
    ) -> QString {
        let banner = banner_url.to_string();
        let prefix = prefix_path.to_string();
        let req = DownloadRequest {
            source: "epic".to_string(),
            app_id: app_id.to_string(),
            display_name: display_name.to_string(),
            banner_url: if banner.is_empty() {
                None
            } else {
                Some(banner)
            },
            install_path: PathBuf::from(install_path.to_string()),
            prefix_path: if prefix.is_empty() {
                None
            } else {
                Some(PathBuf::from(prefix))
            },
            runner_version: runner_version.to_string(),
            temp_dir: None,
            kind: omikuji_core::downloads::DownloadKind::Install,
            destructive_cleanup: true,
            start_paused: false,
        };
        let id = downloads::manager().enqueue(req);
        QString::from(&id)
    }

    fn enqueue_gacha(
        self: Pin<&mut Self>,
        manifest_id: &QString,
        edition_id: &QString,
        voices_csv: &QString,
        display_name: &QString,
        install_path: &QString,
        runner_version: &QString,
        prefix_path: &QString,
        temp_path: &QString,
    ) -> QString {
        use omikuji_core::gachas::{manifest as gm, strategies};

        let mid = manifest_id.to_string();
        let Some(manifest) = gm::find(&mid) else {
            tracing::error!("manifest '{}' not found", mid);
            return QString::default();
        };
        let eid = edition_id.to_string();
        let voices: Vec<String> = voices_csv
            .to_string()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let prefix = prefix_path.to_string();
        let temp = temp_path.to_string();

        let req = match strategies::build_install_request(
            &manifest,
            &eid,
            &voices,
            display_name.to_string(),
            PathBuf::from(install_path.to_string()),
            if prefix.is_empty() {
                None
            } else {
                Some(PathBuf::from(prefix))
            },
            runner_version.to_string(),
            if temp.trim().is_empty() {
                None
            } else {
                Some(PathBuf::from(temp))
            },
        ) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("build request failed: {}", e);
                return QString::default();
            }
        };
        let id = downloads::manager().enqueue(req);
        QString::from(&id)
    }

    fn pause(self: Pin<&mut Self>, id: &QString) {
        downloads::manager().pause(&id.to_string());
    }

    fn resume(self: Pin<&mut Self>, id: &QString) {
        downloads::manager().resume(&id.to_string());
    }

    fn cancel(self: Pin<&mut Self>, id: &QString) {
        downloads::manager().cancel(&id.to_string());
    }

    fn retry(self: Pin<&mut Self>, id: &QString) {
        downloads::manager().retry(&id.to_string());
    }

    fn dismiss(self: Pin<&mut Self>, id: &QString) {
        downloads::manager().dismiss(&id.to_string());
    }

    fn drain_events(mut self: Pin<&mut Self>) {
        let events = downloads::manager().take_events();
        if events.is_empty() {
            return;
        }

        for ev in events {
            match ev {
                DownloadEvent::Added(id) => {
                    if let Some(entry) = downloads::manager().get(&id) {
                        let row = self.entries.len() as i32;
                        self.as_mut()
                            .begin_insert_rows(&QModelIndex::default(), row, row);
                        self.as_mut().rust_mut().get_mut().entries.push(entry);
                        let new_count = self.entries.len() as i32;
                        self.as_mut().set_count(new_count);
                        self.as_mut().end_insert_rows();
                    }
                }
                DownloadEvent::StatusChanged(id, status) => {
                    if let Some(idx) = self.entries.iter().position(|e| e.id == id) {
                        self.as_mut().rust_mut().get_mut().entries[idx].status = status;
                        self.as_mut().notify_row_changed(idx as i32);
                    }
                }
                DownloadEvent::Progress {
                    id,
                    progress,
                    bytes_downloaded,
                    bytes_total,
                    speed_bps,
                } => {
                    if let Some(idx) = self.entries.iter().position(|e| e.id == id) {
                        let entry = &mut self.as_mut().rust_mut().get_mut().entries[idx];
                        entry.progress = progress;
                        entry.bytes_downloaded = bytes_downloaded;
                        if bytes_total > 0 {
                            entry.bytes_total = bytes_total;
                        }
                        entry.speed_bps = speed_bps;
                        self.as_mut().notify_row_changed(idx as i32);
                    }
                }
                DownloadEvent::Completed {
                    id,
                    source,
                    app_id,
                    display_name,
                    install_path,
                    prefix_path,
                    runner_version,
                } => {
                    if let Some(idx) = self.entries.iter().position(|e| e.id == id) {
                        let entry = &mut self.as_mut().rust_mut().get_mut().entries[idx];
                        entry.status = DownloadStatus::Completed;
                        entry.progress = 100.0;
                        self.as_mut().notify_row_changed(idx as i32);
                    }
                    let prefix_str = prefix_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    self.as_mut().download_completed(
                        &QString::from(&id),
                        &QString::from(&source),
                        &QString::from(&app_id),
                        &QString::from(&display_name),
                        &QString::from(&install_path.to_string_lossy().to_string()),
                        &QString::from(&prefix_str),
                        &QString::from(&runner_version),
                    );
                }
                DownloadEvent::Failed(id, err) => {
                    if let Some(idx) = self.entries.iter().position(|e| e.id == id) {
                        let entry = &mut self.as_mut().rust_mut().get_mut().entries[idx];
                        entry.status = DownloadStatus::Failed(err.clone());
                        self.as_mut().notify_row_changed(idx as i32);
                    }
                    self.as_mut()
                        .download_failed(&QString::from(&id), &QString::from(&err));
                }
                DownloadEvent::Removed(id) => {
                    if let Some(idx) = self.entries.iter().position(|e| e.id == id) {
                        let row = idx as i32;
                        self.as_mut()
                            .begin_remove_rows(&QModelIndex::default(), row, row);
                        self.as_mut().rust_mut().get_mut().entries.remove(idx);
                        let new_count = self.entries.len() as i32;
                        self.as_mut().set_count(new_count);
                        self.as_mut().end_remove_rows();
                    }
                }
            }
        }

        let prev_hero = self.hero_id.to_string();
        let c = recompute(&self.entries, &prev_hero);
        self.as_mut().set_active_count(c.active);
        self.as_mut().set_completed_count(c.completed);
        self.as_mut().set_running_count(c.running);
        self.as_mut().set_queued_count(c.queued);
        self.as_mut().set_failed_count(c.failed);
        self.as_mut().set_hero_id(QString::from(&c.hero_id));
        self.as_mut().state_changed();
    }

    fn epic_state_json(&self) -> QString {
        self.source_state_json("epic")
    }

    fn gog_state_json(&self) -> QString {
        self.source_state_json("gog")
    }

    fn source_state_json(&self, source: &str) -> QString {
        let mut map = serde_json::Map::new();
        for e in self.entries.iter().filter(|e| e.source == source) {
            if e.status.is_active() {
                map.insert(
                    e.app_id.clone(),
                    serde_json::json!({
                        "status": status_label(&e.status),
                        "progress": e.progress,
                    }),
                );
            }
        }
        QString::from(&serde_json::Value::Object(map).to_string())
    }

    fn active_for_app_id(&self, app_id: &QString) -> QString {
        let needle = app_id.to_string();
        if needle.is_empty() {
            return QString::from("");
        }
        let prefix = format!("{}:", needle);

        let hit = self.entries.iter().find(|e| {
            let active = e.status.is_active();
            if !active {
                return false;
            }
            e.app_id == needle || e.app_id.starts_with(&prefix)
        });

        let Some(e) = hit else {
            return QString::from("");
        };

        let payload = serde_json::json!({
            "id": e.id,
            "status": status_label(&e.status),
            "progress": e.progress,
            "kind": kind_label(&e.kind),
        });
        QString::from(&payload.to_string())
    }

    fn speed_history_json(&self) -> QString {
        QString::from(&downloads::io_stats::history_json())
    }
}
