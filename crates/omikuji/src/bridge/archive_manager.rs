// fresh runtime per op since nesting one under #[tokio::main] panics
// also check drain_events

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use omikuji_core::archive_source::{self, InstallChannel, ReleaseInfo};
use omikuji_core::components_config::{self, ArchiveSource, SourceCategory};
use omikuji_core::runners;
use omikuji_core::store::steam::local as steam_local;
use omikuji_core::{anyhow, archive};
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

        #[qsignal]
        #[cxx_name = "advisedRunnerReady"]
        fn advised_runner_ready(
            self: Pin<&mut ArchiveManagerBridge>,
            json: QString,
            error: QString,
        );

        #[qinvokable]
        #[cxx_name = "listSources"]
        fn list_sources(self: &ArchiveManagerBridge, category: QString) -> QString;

        #[qinvokable]
        #[cxx_name = "listInstalled"]
        fn list_installed(
            self: &ArchiveManagerBridge,
            category: QString,
            source: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "fetchVersions"]
        fn fetch_versions(self: Pin<&mut ArchiveManagerBridge>, category: QString, source: QString);

        #[qinvokable]
        #[cxx_name = "fetchAdvisedRunner"]
        fn fetch_advised_runner(self: Pin<&mut ArchiveManagerBridge>, link: QString);

        #[qinvokable]
        #[cxx_name = "installAdvisedRunner"]
        fn install_advised_runner(
            self: Pin<&mut ArchiveManagerBridge>,
            link: QString,
            asset_name: QString,
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
        #[cxx_name = "normalizeReleasesUrl"]
        fn normalize_releases_url(self: &ArchiveManagerBridge, link: QString) -> QString;

        #[qinvokable]
        #[cxx_name = "addSource"]
        fn add_source(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            source_json: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "updateSource"]
        fn update_source(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            name: QString,
            source_json: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "removeSource"]
        fn remove_source(
            self: Pin<&mut ArchiveManagerBridge>,
            category: QString,
            name: QString,
        ) -> QString;

        #[qsignal]
        #[cxx_name = "moveToSteamDone"]
        fn move_to_steam_done(self: Pin<&mut ArchiveManagerBridge>, tag: QString, error: QString);

        #[qinvokable]
        #[cxx_name = "listSteamRoots"]
        fn list_steam_roots(self: &ArchiveManagerBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "moveToSteamAt"]
        fn move_to_steam_at(
            self: Pin<&mut ArchiveManagerBridge>,
            runner_dir: QString,
            roots_json: QString,
        );

        #[qinvokable]
        #[cxx_name = "installLatest"]
        fn install_latest(self: Pin<&mut ArchiveManagerBridge>, source: QString);

        #[qinvokable]
        #[cxx_name = "steamActionFor"]
        fn steam_action_for(self: &ArchiveManagerBridge, runner_dir: QString) -> QString;

        #[qinvokable]
        #[cxx_name = "steamLinkedRoots"]
        fn steam_linked_roots(self: &ArchiveManagerBridge, runner_dir: QString) -> QString;

        #[qinvokable]
        #[cxx_name = "setSteamLinks"]
        fn set_steam_links(
            self: &ArchiveManagerBridge,
            runner_dir: QString,
            roots_json: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "setDllPackPrefixInstallVersion"]
        fn set_dll_pack_prefix_install_version(
            self: Pin<&mut ArchiveManagerBridge>,
            source: QString,
            tag: QString,
        );

        #[qinvokable]
        #[cxx_name = "foundRunners"]
        fn found_runners(self: &ArchiveManagerBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "installedRunnerPath"]
        fn installed_runner_path(
            self: &ArchiveManagerBridge,
            source: QString,
            name: QString,
        ) -> QString;

        #[qinvokable]
        #[cxx_name = "deleteRunnerAt"]
        fn delete_runner_at(self: &ArchiveManagerBridge, path: QString) -> QString;

        #[qinvokable]
        #[cxx_name = "drainEvents"]
        fn drain_events(self: Pin<&mut ArchiveManagerBridge>);
    }
}

#[derive(Default)]
pub struct ArchiveManagerRust;

fn lookup(category: &str, name: &str) -> Option<(SourceCategory, ArchiveSource)> {
    let category: SourceCategory = category.parse().ok()?;
    Some((category, category.find(name)?))
}

fn resolve_release(link: &str) -> Result<(runners::AdvisedRunner, ReleaseInfo), String> {
    let advised =
        runners::resolve_advised(link).ok_or_else(|| format!("unusable runner link: {}", link))?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let releases = rt
        .block_on(archive_source::fetch_versions(&advised.source))
        .map_err(|e| format!("{:#}", e))?;
    let release = releases
        .into_iter()
        .find(|r| r.tag == advised.tag)
        .ok_or_else(|| format!("no release tagged {}", advised.tag))?;
    Ok((advised, release))
}

fn sources_to_json(sources: &[ArchiveSource]) -> String {
    serde_json::to_string(sources).unwrap_or_else(|_| "[]".into())
}

impl qobject::ArchiveManagerBridge {
    fn list_sources(&self, category: QString) -> QString {
        let sources = category
            .to_string()
            .parse()
            .map(SourceCategory::sources)
            .unwrap_or_default();
        QString::from(&sources_to_json(&sources))
    }

    fn list_installed(&self, category: QString, source: QString) -> QString {
        let Some((category, src)) = lookup(&category.to_string(), &source.to_string()) else {
            return QString::from("[]");
        };
        let installed = category.list_installed(&src);
        QString::from(&serde_json::to_string(&installed).unwrap_or_else(|_| "[]".into()))
    }

    fn fetch_versions(mut self: Pin<&mut Self>, category: QString, source: QString) {
        let cat = category.to_string();
        let name = source.to_string();
        let Some((_, src)) = lookup(&cat, &name) else {
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
            let _ = qt_thread.queue(move |mut this: Pin<&mut qobject::ArchiveManagerBridge>| {
                match result {
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
                }
            });
        });
    }

    fn fetch_advised_runner(mut self: Pin<&mut Self>, link: QString) {
        let link_s = link.to_string();
        let qt = self.as_mut().qt_thread();
        thread::spawn(move || {
            let emit = |json: String, error: String| {
                let _ = qt.queue(move |bridge| {
                    bridge.advised_runner_ready(QString::from(&json), QString::from(&error));
                });
            };
            let (advised, release) = match resolve_release(&link_s) {
                Ok(v) => v,
                Err(e) => return emit(String::new(), e),
            };
            let installed = archive_source::list_installed(&advised.source, &advised.dest_root());
            let stems: Vec<String> = release
                .assets
                .iter()
                .map(|a| archive::stem(&a.name).to_string())
                .collect();
            emit(
                serde_json::json!({
                    "tag": advised.tag,
                    "assets": release.assets,
                    "installedDirs": installed,
                    "steamProtonDirs": runners::steam_proton_dirs_by_name(&stems),
                })
                .to_string(),
                String::new(),
            );
        });
    }

    fn install_advised_runner(mut self: Pin<&mut Self>, link: QString, asset_name: QString) {
        let link_s = link.to_string();
        let wanted = asset_name.to_string();
        let qt = self.as_mut().qt_thread();
        thread::spawn(move || {
            let (advised, mut release) = match resolve_release(&link_s) {
                Ok(v) => v,
                Err(e) => {
                    let _ = qt.queue(move |bridge| {
                        bridge.install_failed(
                            QString::from(InstallChannel::Runners.as_str()),
                            QString::from(""),
                            QString::from(""),
                            QString::from(&e),
                        );
                    });
                    return;
                }
            };
            if !wanted.is_empty()
                && let Some(asset) = release.assets.iter().find(|a| a.name == wanted).cloned()
            {
                release.asset_name = asset.name;
                release.asset_url = asset.url;
                release.asset_size = asset.size;
            }
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => return,
            };
            let _ = rt.block_on(archive_source::install_version(
                InstallChannel::Runners,
                &advised.source,
                &release,
                &advised.dest_root(),
            ));
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
        let Some((category, src)) = lookup(&cat, &name) else {
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

        thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => return,
            };
            let _ = rt.block_on(category.install_version(&src, &release));
        });
    }

    fn delete_version(mut self: Pin<&mut Self>, category: QString, source: QString, tag: QString) {
        let cat = category.to_string();
        let name = source.to_string();
        let tag_s = tag.to_string();
        let Some((category, src)) = lookup(&cat, &name) else {
            return;
        };
        if let Err(e) = category.delete_version(&src, &tag_s) {
            self.as_mut().install_failed(
                QString::from(&cat),
                QString::from(&name),
                QString::from(&tag_s),
                QString::from(&format!("delete: {:#}", e)),
            );
        }
    }

    fn normalize_releases_url(&self, link: QString) -> QString {
        QString::from(&archive_source::normalize_releases_url(&link.to_string()))
    }

    fn edit_sources(
        mut self: Pin<&mut Self>,
        category: &QString,
        edit: impl FnOnce(SourceCategory) -> anyhow::Result<()>,
    ) -> QString {
        match category.to_string().parse().and_then(edit) {
            Ok(()) => {
                self.as_mut().sources_changed();
                QString::from("")
            }
            Err(e) => QString::from(&format!("{:#}", e)),
        }
    }

    fn add_source(self: Pin<&mut Self>, category: QString, source_json: QString) -> QString {
        let source: ArchiveSource = match serde_json::from_str(&source_json.to_string()) {
            Ok(s) => s,
            Err(e) => return QString::from(&format!("source parse: {}", e)),
        };
        self.edit_sources(&category, |category| {
            components_config::add_source(category, source)
        })
    }

    fn update_source(
        self: Pin<&mut Self>,
        category: QString,
        name: QString,
        source_json: QString,
    ) -> QString {
        let source: ArchiveSource = match serde_json::from_str(&source_json.to_string()) {
            Ok(s) => s,
            Err(e) => return QString::from(&format!("source parse: {}", e)),
        };
        self.edit_sources(&category, |category| {
            components_config::update_source(category, &name.to_string(), source)
        })
    }

    fn remove_source(self: Pin<&mut Self>, category: QString, name: QString) -> QString {
        self.edit_sources(&category, |category| {
            components_config::remove_source(category, &name.to_string())
        })
    }

    fn list_steam_roots(&self) -> QString {
        let roots: Vec<(String, String)> = steam_local::steam_install_roots()
            .into_iter()
            .map(|(label, path)| (label, path.to_string_lossy().into_owned()))
            .collect();
        QString::from(&serde_json::to_string(&roots).unwrap_or_else(|_| "[]".into()))
    }

    fn install_latest(mut self: Pin<&mut Self>, source: QString) {
        let name = source.to_string();
        let fail = |bridge: Pin<&mut qobject::ArchiveManagerBridge>, name: &str, err: &str| {
            bridge.install_failed(
                QString::from(InstallChannel::RunnersLatest.as_str()),
                QString::from(name),
                QString::from(""),
                QString::from(err),
            );
        };
        let Some(src) = SourceCategory::Runners.find(&name) else {
            fail(self.as_mut(), &name, &format!("unknown source: {}", name));
            return;
        };
        let qt_thread = self.as_mut().qt_thread();
        thread::spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("{}", e))
                .and_then(|rt| {
                    rt.block_on(runners::ensure_latest(&src))
                        .map_err(|e| format!("{:#}", e))
                });
            let Err(err) = result else {
                return;
            };
            let _ = qt_thread.queue(move |mut this: Pin<&mut qobject::ArchiveManagerBridge>| {
                fail(this.as_mut(), &name, &err);
            });
        });
    }

    fn steam_action_for(&self, runner_dir: QString) -> QString {
        let dir = std::path::PathBuf::from(runner_dir.to_string());
        QString::from(runners::steam_action(&dir).as_str())
    }

    fn steam_linked_roots(&self, runner_dir: QString) -> QString {
        let dir = std::path::PathBuf::from(runner_dir.to_string());
        let roots: Vec<String> = runners::steam_link_roots(&dir)
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        QString::from(&serde_json::to_string(&roots).unwrap_or_else(|_| "[]".into()))
    }

    fn set_steam_links(&self, runner_dir: QString, roots_json: QString) -> QString {
        let dir = std::path::PathBuf::from(runner_dir.to_string());
        let roots: Vec<std::path::PathBuf> =
            serde_json::from_str::<Vec<String>>(&roots_json.to_string())
                .unwrap_or_default()
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect();
        match runners::set_steam_links(&dir, &roots) {
            Ok(()) => QString::from(""),
            Err(e) => QString::from(&format!("{:#}", e)),
        }
    }

    fn move_to_steam_at(mut self: Pin<&mut Self>, runner_dir: QString, roots_json: QString) {
        let dir = std::path::PathBuf::from(runner_dir.to_string());
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let roots: Vec<std::path::PathBuf> =
            serde_json::from_str::<Vec<String>>(&roots_json.to_string())
                .unwrap_or_default()
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect();
        let qt_thread = self.as_mut().qt_thread();
        thread::spawn(move || {
            let err = runners::move_to_steam_dir(&dir, &roots)
                .err()
                .map(|e| format!("{:#}", e))
                .unwrap_or_default();
            let _ = qt_thread.queue(move |mut this: Pin<&mut qobject::ArchiveManagerBridge>| {
                this.as_mut()
                    .move_to_steam_done(QString::from(&name), QString::from(&err));
            });
        });
    }

    fn set_dll_pack_prefix_install_version(self: Pin<&mut Self>, source: QString, tag: QString) {
        let name = source.to_string();
        if let Err(e) = components_config::set_prefix_install_version(&name, &tag.to_string()) {
            tracing::error!("save failed for {}: {}", name, e);
        }
    }

    fn found_runners(&self) -> QString {
        QString::from(
            &serde_json::to_string(&runners::found_runners()).unwrap_or_else(|_| "[]".into()),
        )
    }

    fn installed_runner_path(&self, source: QString, name: QString) -> QString {
        let Some(src) = SourceCategory::Runners.find(&source.to_string()) else {
            return QString::from("");
        };
        let path = runners::source_root(&src).join(name.to_string());
        if path.is_dir() {
            QString::from(&path.to_string_lossy().into_owned())
        } else {
            QString::from("")
        }
    }

    fn delete_runner_at(&self, path: QString) -> QString {
        let dir = std::path::PathBuf::from(path.to_string());
        match runners::delete_found_runner(&dir) {
            Ok(_) => QString::from(""),
            Err(e) => QString::from(&format!("{:#}", e)),
        }
    }

    fn drain_events(mut self: Pin<&mut Self>) {
        for ev in archive_source::drain_events() {
            match ev {
                archive_source::ArchiveEvent::Started {
                    channel,
                    source,
                    tag,
                } => {
                    self.as_mut().install_started(
                        QString::from(channel.as_str()),
                        QString::from(&source),
                        QString::from(&tag),
                    );
                }
                archive_source::ArchiveEvent::Progress {
                    channel,
                    source,
                    tag,
                    phase,
                    percent,
                } => {
                    self.as_mut().install_progress(
                        QString::from(channel.as_str()),
                        QString::from(&source),
                        QString::from(&tag),
                        QString::from(&phase),
                        percent,
                    );
                }
                archive_source::ArchiveEvent::Completed {
                    channel,
                    source,
                    tag,
                    install_dir,
                } => {
                    self.as_mut().install_completed(
                        QString::from(channel.as_str()),
                        QString::from(&source),
                        QString::from(&tag),
                        QString::from(&install_dir),
                    );
                }
                archive_source::ArchiveEvent::Failed {
                    channel,
                    source,
                    tag,
                    error,
                } => {
                    self.as_mut().install_failed(
                        QString::from(channel.as_str()),
                        QString::from(&source),
                        QString::from(&tag),
                        QString::from(&error),
                    );
                }
            }
        }
    }
}
