// unified bridge for runners and dll packs. both use the same settings-driven fetch pipeline
// (see core::archive_source), so theres no reason to doubel the qobject surface; 
// the category argument ("runners" / "dll_packs") picks the right source list and install target.
//
// async ops run on a detached os thread with a fresh tokio runtime. nesting a runtime from a cxx-qt invokable panics becuase main is #[tokio::main]; 
// results flow back as events on the archive_source queue, drained by drainEvents on a qml timer.

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use omikuji_core::archive_source::{self, ReleaseInfo};
use omikuji_core::components_config::{self, ArchiveSource};
use omikuji_core::dll_packs;
use omikuji_core::runners;
use std::pin::Pin;
use std::thread;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type ArchiveManagerBridge = super::ArchiveManagerRust;
    }

    // enable qt_thread(), required for async fetch results to marshal back to the ui thread
    impl cxx_qt::Threading for ArchiveManagerBridge {}

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "installStarted"]
        fn install_started(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            tag: QString,
        );

        #[qsignal]
        #[cxx_name = "installProgress"]
        fn install_progress(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            tag: QString,
            phase: QString,
            percent: f64,
        );

        #[qsignal]
        #[cxx_name = "installCompleted"]
        fn install_completed(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            tag: QString,
            install_dir: QString,
        );

        #[qsignal]
        #[cxx_name = "installFailed"]
        fn install_failed(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            tag: QString,
            error: QString,
        );

        #[qsignal]
        #[cxx_name = "versionsReady"]
        fn versions_ready(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            json: QString,
        );

        #[qsignal]
        #[cxx_name = "versionsFailed"]
        fn versions_failed(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            error: QString,
        );

        #[qsignal]
        #[cxx_name = "sourcesChanged"]
        fn sources_changed(self: Pin<&mut ArchiveManagerBridge>);

        #[qinvokable]
        #[cxx_name = "listRunners"]
        fn list_runners(self: &ArchiveManagerBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "listDllPacks"]
        fn list_dll_packs(self: &ArchiveManagerBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "listInstalled"]
        fn list_installed(
            self: &ArchiveManagerBridge,
            category: QString,
            source: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "fetchVersions"]
        fn fetch_versions(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
        );

        // release_json is a single ReleaseInfo object (from a previous versionsReady payload).
        // passing it rather than just the tag avoids a second list round-trip to resolve asset_url.
        #[qinvokable]
        #[cxx_name = "installVersion"]
        fn install_version(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            release_json: QString,
        );

        #[qinvokable]
        #[cxx_name = "deleteVersion"]
        fn delete_version(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source: QString,
            tag: QString,
        );

        #[qinvokable]
        #[cxx_name = "addSource"]
        fn add_source(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source_json: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "removeSource"]
        fn remove_source(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            name: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "dllPackActiveVersion"]
        fn dll_pack_active_version(
            self: &ArchiveManagerBridge,
            source: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "setDllPackActiveVersion"]
        fn set_dll_pack_active_version(
            self: Pin<&mut ArchiveManagerBridge>,
            source: QString,
            tag: QString,
        );

        #[qinvokable]
        #[cxx_name = "drainEvents"]
        fn drain_events(self: Pin<&mut ArchiveManagerBridge>);
    }
}

#[derive(Default)]
pub struct ArchiveManagerRust;

fn sources_for(category: &str) -> Vec<ArchiveSource> {
    match category {
        "runners" => runners::list_sources(),
        "dll_packs" => dll_packs::list_sources(),
        _ => Vec::new(),
    }
}

// the ui speaks "dll_packs", components.toml calls them layers
fn core_category(category: &str) -> &str {
    if category == "dll_packs" { "layers" } else { category }
}

fn source_lookup(category: &str, name: &str) -> Option<ArchiveSource> {
    sources_for(category).into_iter().find(|s| s.name == name)
}

fn sources_to_json(sources: &[ArchiveSource]) -> String {
    serde_json::to_string(sources).unwrap_or_else(|_| "[]".into())
}

impl qobject::ArchiveManagerBridge {
    fn list_runners(&self) -> QString {
        QString::from(&sources_to_json(&runners::list_sources()))
    }

    fn list_dll_packs(&self) -> QString {
        QString::from(&sources_to_json(&dll_packs::list_sources()))
    }

    fn list_installed(&self, category: QString, source: QString) -> QString {
        let cat = category.to_string();
        let name = source.to_string();
        let Some(src) = source_lookup(&cat, &name) else {
            return QString::from("[]");
        };
        let installed = match cat.as_str() {
            "runners" => runners::list_installed(&src),
            "dll_packs" => dll_packs::list_installed(&src),
            _ => Vec::new(),
        };
        QString::from(&serde_json::to_string(&installed).unwrap_or_else(|_| "[]".into()))
    }

    fn fetch_versions(mut self: Pin<&mut Self>, category: QString, source: QString) {
        let cat = category.to_string();
        let name = source.to_string();
        let Some(src) = source_lookup(&cat, &name) else {
            self.as_mut().versions_failed(
                QString::from(&cat),
                QString::from(&name),
                QString::from(&format!("unknown source: {}/{}", cat, name)),
            );
            return;
        };
        let qt_thread = self.as_mut().qt_thread();
        thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let err = format!("{}", e);
                    let _ = qt_thread.queue(
                        move |mut this: Pin<&mut qobject::ArchiveManagerBridge>| {
                            this.as_mut().versions_failed(
                                QString::from(&cat),
                                QString::from(&name),
                                QString::from(&err),
                            );
                        },
                    );
                    return;
                }
            };
            let result = rt.block_on(archive_source::fetch_versions(&src));
            let _ = qt_thread.queue(
                move |mut this: Pin<&mut qobject::ArchiveManagerBridge>| match result {
                    Ok(list) => {
                        let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".into());
                        this.as_mut().versions_ready(
                            QString::from(&cat),
                            QString::from(&name),
                            QString::from(&json),
                        );
                    }
                    Err(e) => {
                        this.as_mut().versions_failed(
                            QString::from(&cat),
                            QString::from(&name),
                            QString::from(&format!("{:#}", e)),
                        );
                    }
                },
            );
        });
    }

    fn install_version(
        mut self: Pin<&mut Self>,
        category: QString,
        source: QString,
        release_json: QString,
    ) {
        let cat = category.to_string();
        let name = source.to_string();
        let Some(src) = source_lookup(&cat, &name) else {
            self.as_mut().install_failed(
                QString::from(&cat),
                QString::from(&name),
                QString::from(""),
                QString::from(&format!("unknown source: {}/{}", cat, name)),
            );
            return;
        };
        let release: ReleaseInfo = match serde_json::from_str(&release_json.to_string()) {
            Ok(r) => r,
            Err(e) => {
                self.as_mut().install_failed(
                    QString::from(&cat),
                    QString::from(&name),
                    QString::from(""),
                    QString::from(&format!("release_json parse: {}", e)),
                );
                return;
            }
        };

        let dest_root = match cat.as_str() {
            "runners" => runners::source_root(&src),
            "dll_packs" => dll_packs::source_root(&src),
            other => {
                self.as_mut().install_failed(
                    QString::from(&cat),
                    QString::from(&name),
                    QString::from(&release.tag),
                    QString::from(&format!("unknown category: {}", other)),
                );
                return;
            }
        };
        let cat_for_thread = cat.clone();
        thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => return,
            };
            let _ = rt.block_on(archive_source::install_version(
                &cat_for_thread,
                &src,
                &release,
                &dest_root,
            ));
        });
    }

    fn delete_version(
        mut self: Pin<&mut Self>,
        category: QString,
        source: QString,
        tag: QString,
    ) {
        let cat = category.to_string();
        let name = source.to_string();
        let tag_s = tag.to_string();
        let Some(src) = source_lookup(&cat, &name) else {
            return;
        };
        let res = match cat.as_str() {
            "runners" => runners::delete_version(&src, &tag_s),
            "dll_packs" => dll_packs::delete_version(&src, &tag_s),
            _ => Ok(()),
        };
        if let Err(e) = res {
            self.as_mut().install_failed(
                QString::from(&cat),
                QString::from(&name),
                QString::from(&tag_s),
                QString::from(&format!("delete: {:#}", e)),
            );
        }
    }

    fn add_source(mut self: Pin<&mut Self>, category: QString, source_json: QString) -> QString {
        let source: ArchiveSource = match serde_json::from_str(&source_json.to_string()) {
            Ok(s) => s,
            Err(e) => return QString::from(&format!("source parse: {}", e)),
        };
        match components_config::add_source(core_category(&category.to_string()), source) {
            Ok(_) => {
                self.as_mut().sources_changed();
                QString::from("")
            }
            Err(e) => QString::from(&format!("{:#}", e)),
        }
    }

    fn remove_source(mut self: Pin<&mut Self>, category: QString, name: QString) -> QString {
        match components_config::remove_source(
            core_category(&category.to_string()),
            &name.to_string(),
        ) {
            Ok(_) => {
                self.as_mut().sources_changed();
                QString::from("")
            }
            Err(e) => QString::from(&format!("{:#}", e)),
        }
    }

    fn dll_pack_active_version(&self, source: QString) -> QString {
        let name = source.to_string();
        QString::from(&components_config::active_version(&name))
    }

    fn set_dll_pack_active_version(
        self: Pin<&mut Self>,
        source: QString,
        tag: QString,
    ) {
        let name = source.to_string();
        let tag_s = tag.to_string();
        if let Err(e) = components_config::set_active_version(&name, &tag_s) {
            tracing::error!("save failed for {}: {}", name, e);
        }
    }

    fn drain_events(mut self: Pin<&mut Self>) {
        for ev in archive_source::drain_events() {
            match ev {
                archive_source::ArchiveEvent::Started { category, source, tag } => {
                    self.as_mut().install_started(
                        QString::from(&category),
                        QString::from(&source),
                        QString::from(&tag),
                    );
                }
                archive_source::ArchiveEvent::Progress {
                    category, source, tag, phase, percent,
                } => {
                    self.as_mut().install_progress(
                        QString::from(&category),
                        QString::from(&source),
                        QString::from(&tag),
                        QString::from(&phase),
                        percent,
                    );
                }
                archive_source::ArchiveEvent::Completed {
                    category, source, tag, install_dir,
                } => {
                    self.as_mut().install_completed(
                        QString::from(&category),
                        QString::from(&source),
                        QString::from(&tag),
                        QString::from(&install_dir),
                    );
                }
                archive_source::ArchiveEvent::Failed {
                    category, source, tag, error,
                } => {
                    self.as_mut().install_failed(
                        QString::from(&category),
                        QString::from(&source),
                        QString::from(&tag),
                        QString::from(&error),
                    );
                }
            }
        }
    }
}
