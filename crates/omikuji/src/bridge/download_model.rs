
#![allow(clippy::too_many_arguments)]

use std::path::PathBuf;
use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};

use omikuji_core::downloads::{
    self, DownloadEntry, DownloadEvent, DownloadRequest, DownloadStatus,
};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray =
            cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(i32, count)]
        #[qproperty(i32, active_count, cxx_name = "activeCount")]
        #[qproperty(i32, completed_count, cxx_name = "completedCount")]
        type DownloadModel = super::DownloadModelRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn download_completed(
            self: Pin<&mut DownloadModel>,
            id: &QString,
            source: &QString,
            app_id: &QString,
            display_name: &QString,
            install_path: &QString,
            prefix_path: &QString,
            runner_version: &QString,
        );

        #[qsignal]
        fn download_failed(self: Pin<&mut DownloadModel>, id: &QString, error: &QString);

        #[cxx_name = "rowCount"]
        #[cxx_override]
        fn row_count(self: &DownloadModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &DownloadModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_name = "roleNames"]
        #[cxx_override]
        fn role_names(self: &DownloadModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        fn enqueue_epic(
            self: Pin<&mut DownloadModel>,
            app_id: &QString,
            display_name: &QString,
            banner_url: &QString,
            install_path: &QString,
            prefix_path: &QString,
            runner_version: &QString,
        ) -> QString;

        #[qinvokable]
        fn enqueue_gacha(
            self: Pin<&mut DownloadModel>,
            manifest_id: &QString,
            edition_id: &QString,
            voices_csv: &QString,
            display_name: &QString,
            install_path: &QString,
            runner_version: &QString,
            prefix_path: &QString,
            temp_path: &QString,
        ) -> QString;

        #[qinvokable]
        fn pause(self: Pin<&mut DownloadModel>, id: &QString);
        #[qinvokable]
        fn resume(self: Pin<&mut DownloadModel>, id: &QString);
        #[qinvokable]
        fn cancel(self: Pin<&mut DownloadModel>, id: &QString);

        #[qinvokable]
        fn retry(self: Pin<&mut DownloadModel>, id: &QString);

        #[qinvokable]
        fn dismiss(self: Pin<&mut DownloadModel>, id: &QString);

        #[qinvokable]
        fn drain_events(self: Pin<&mut DownloadModel>);

        #[qinvokable]
        fn epic_state_json(self: &DownloadModel) -> QString;

        #[qinvokable]
        fn gog_state_json(self: &DownloadModel) -> QString;

        // match is "equal or prefix-colon" so "zzz:global" also matches
        // "zzz:global:en-us,ja-jp" (hoyo encodes voice locales in app_id)
        #[qinvokable]
        fn active_for_app_id(self: &DownloadModel, app_id: &QString) -> QString;

        #[qsignal]
        fn state_changed(self: Pin<&mut DownloadModel>);
    }

    unsafe extern "RustQt" {
        #[cxx_name = "beginInsertRows"]
        #[inherit]
        fn begin_insert_rows(self: Pin<&mut DownloadModel>, parent: &QModelIndex, first: i32, last: i32);

        #[cxx_name = "endInsertRows"]
        #[inherit]
        fn end_insert_rows(self: Pin<&mut DownloadModel>);

        #[cxx_name = "beginRemoveRows"]
        #[inherit]
        fn begin_remove_rows(self: Pin<&mut DownloadModel>, parent: &QModelIndex, first: i32, last: i32);

        #[cxx_name = "endRemoveRows"]
        #[inherit]
        fn end_remove_rows(self: Pin<&mut DownloadModel>);

        #[cxx_name = "dataChanged"]
        #[inherit]
        fn data_changed(
            self: Pin<&mut DownloadModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
        );

        #[cxx_name = "index"]
        #[inherit]
        fn index_for(self: &DownloadModel, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
    }
}

const ROLE_ID: i32 = 0x0200;
const ROLE_SOURCE: i32 = 0x0201;
const ROLE_APP_ID: i32 = 0x0202;
const ROLE_DISPLAY_NAME: i32 = 0x0203;
const ROLE_BANNER: i32 = 0x0204;
const ROLE_STATUS: i32 = 0x0205;
const ROLE_PROGRESS: i32 = 0x0206;
const ROLE_SPEED: i32 = 0x0207;
const ROLE_BYTES_DL: i32 = 0x0208;
const ROLE_BYTES_TOTAL: i32 = 0x0209;
const ROLE_ERROR: i32 = 0x020A;

pub struct DownloadModelRust {
    entries: Vec<DownloadEntry>,
    count: i32,
    active_count: i32,
    completed_count: i32,
}

impl Default for DownloadModelRust {
    fn default() -> Self {
        let entries = downloads::manager().list();
        let count = entries.len() as i32;
        let active_count = entries
            .iter()
            .filter(|e| {
                matches!(
                    e.status,
                    DownloadStatus::Queued
                        | DownloadStatus::Starting
                        | DownloadStatus::Downloading
                        | DownloadStatus::Extracting
                        | DownloadStatus::Paused
                )
            })
            .count() as i32;
        let completed_count = entries
            .iter()
            .filter(|e| matches!(e.status, DownloadStatus::Completed))
            .count() as i32;
        Self {
            entries,
            count,
            active_count,
            completed_count,
        }
    }
}

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

fn recompute_active(entries: &[DownloadEntry]) -> i32 {
    entries
        .iter()
        .filter(|e| {
            matches!(
                e.status,
                DownloadStatus::Queued
                    | DownloadStatus::Starting
                    | DownloadStatus::Downloading
                    | DownloadStatus::Paused
            )
        })
        .count() as i32
}

fn recompute_completed(entries: &[DownloadEntry]) -> i32 {
    entries
        .iter()
        .filter(|e| matches!(e.status, DownloadStatus::Completed))
        .count() as i32
}

impl qobject::DownloadModel {
    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.entries.len() as i32
    }

    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut h = QHash::<QHashPair_i32_QByteArray>::default();
        h.insert_clone(&ROLE_ID, &QByteArray::from("id"));
        h.insert_clone(&ROLE_SOURCE, &QByteArray::from("source"));
        h.insert_clone(&ROLE_APP_ID, &QByteArray::from("appId"));
        h.insert_clone(&ROLE_DISPLAY_NAME, &QByteArray::from("displayName"));
        h.insert_clone(&ROLE_BANNER, &QByteArray::from("banner"));
        h.insert_clone(&ROLE_STATUS, &QByteArray::from("status"));
        h.insert_clone(&ROLE_PROGRESS, &QByteArray::from("progress"));
        h.insert_clone(&ROLE_SPEED, &QByteArray::from("speed"));
        h.insert_clone(&ROLE_BYTES_DL, &QByteArray::from("bytesDownloaded"));
        h.insert_clone(&ROLE_BYTES_TOTAL, &QByteArray::from("bytesTotal"));
        h.insert_clone(&ROLE_ERROR, &QByteArray::from("error"));
        h
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let row = index.row() as usize;
        let Some(e) = self.entries.get(row) else {
            return QVariant::default();
        };
        match role {
            ROLE_ID => QVariant::from(&QString::from(&*e.id)),
            ROLE_SOURCE => QVariant::from(&QString::from(&*e.source)),
            ROLE_APP_ID => QVariant::from(&QString::from(&*e.app_id)),
            ROLE_DISPLAY_NAME => QVariant::from(&QString::from(&*e.display_name)),
            ROLE_BANNER => QVariant::from(&QString::from(e.banner_url.as_deref().unwrap_or(""))),
            ROLE_STATUS => QVariant::from(&QString::from(status_label(&e.status))),
            ROLE_PROGRESS => QVariant::from(&e.progress),
            ROLE_SPEED => QVariant::from(&(e.speed_bps as f64)),
            ROLE_BYTES_DL => QVariant::from(&(e.bytes_downloaded as f64)),
            ROLE_BYTES_TOTAL => QVariant::from(&(e.bytes_total as f64)),
            ROLE_ERROR => QVariant::from(&QString::from(&error_text(&e.status))),
            _ => QVariant::default(),
        }
    }

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
            banner_url: if banner.is_empty() { None } else { Some(banner) },
            install_path: PathBuf::from(install_path.to_string()),
            prefix_path: if prefix.is_empty() { None } else { Some(PathBuf::from(prefix)) },
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
            eprintln!("[enqueue_gacha] manifest '{}' not found", mid);
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
            if prefix.is_empty() { None } else { Some(PathBuf::from(prefix)) },
            runner_version.to_string(),
            if temp.trim().is_empty() { None } else { Some(PathBuf::from(temp)) },
        ) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[enqueue_gacha] build request failed: {}", e);
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
                        let parent = QModelIndex::default();
                        let qidx = self.as_ref().index_for(idx as i32, 0, &parent);
                        self.as_mut().data_changed(&qidx, &qidx);
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
                        let parent = QModelIndex::default();
                        let qidx = self.as_ref().index_for(idx as i32, 0, &parent);
                        self.as_mut().data_changed(&qidx, &qidx);
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
                        let parent = QModelIndex::default();
                        let qidx = self.as_ref().index_for(idx as i32, 0, &parent);
                        self.as_mut().data_changed(&qidx, &qidx);
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
                        let parent = QModelIndex::default();
                        let qidx = self.as_ref().index_for(idx as i32, 0, &parent);
                        self.as_mut().data_changed(&qidx, &qidx);
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

        let new_active = recompute_active(&self.entries);
        let new_completed = recompute_completed(&self.entries);
        self.as_mut().set_active_count(new_active);
        self.as_mut().set_completed_count(new_completed);
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
            if matches!(
                e.status,
                DownloadStatus::Queued
                    | DownloadStatus::Starting
                    | DownloadStatus::Downloading
                    | DownloadStatus::Paused
            ) {
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
            let active = matches!(
                e.status,
                omikuji_core::downloads::DownloadStatus::Queued
                    | omikuji_core::downloads::DownloadStatus::Starting
                    | omikuji_core::downloads::DownloadStatus::Downloading
                    | omikuji_core::downloads::DownloadStatus::Extracting
                    | omikuji_core::downloads::DownloadStatus::Patching
                    | omikuji_core::downloads::DownloadStatus::Paused
            );
            if !active {
                return false;
            }
            e.app_id == needle || e.app_id.starts_with(&prefix)
        });

        let Some(e) = hit else {
            return QString::from("");
        };

        let kind = match &e.kind {
            omikuji_core::downloads::DownloadKind::Install => "install",
            omikuji_core::downloads::DownloadKind::Update { .. } => "update",
        };

        let payload = serde_json::json!({
            "id": e.id,
            "status": status_label(&e.status),
            "progress": e.progress,
            "kind": kind,
        });
        QString::from(&payload.to_string())
    }
}
