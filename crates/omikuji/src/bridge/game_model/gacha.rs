use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QModelIndex, QString};

use omikuji_core::library::{Game, Library};

impl super::qobject::GameModel {
    pub fn list_gachas(&self) -> QString {
        let manifests = omikuji_core::gachas::manifest::load_all();
        match serde_json::to_string(&manifests) {
            Ok(s) => QString::from(&s),
            Err(e) => {
                eprintln!("[list_gachas] serialize failed: {}", e);
                QString::from("[]")
            }
        }
    }

    pub fn ensure_gacha_manifests(self: Pin<&mut Self>) {
        let sender = self.as_ref().qt_thread();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("[gachas] couldn't build runtime: {}", e);
                    return;
                }
            };
            let fetched = match rt.block_on(omikuji_core::gachas::remote::ensure_all_fetched()) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[gachas] fetch failed: {}", e);
                    omikuji_core::notifications::warning(
                        "Gachas",
                        "Couldn't fetch manifests. Existing cached games still work.",
                    );
                    0
                }
            };
            let _ = sender.queue(move |mut m: Pin<&mut super::qobject::GameModel>| {
                m.as_mut().gacha_manifests_ready(fetched as i32);
            });
        });
    }

    pub fn get_gacha_manifest(&self, manifest_id: &QString) -> QString {
        let id = manifest_id.to_string();
        match omikuji_core::gachas::manifest::find(&id) {
            Some(m) => match serde_json::to_string(&m) {
                Ok(s) => QString::from(&s),
                Err(e) => {
                    eprintln!("[get_gacha_manifest] serialize failed: {}", e);
                    QString::default()
                }
            },
            None => QString::default(),
        }
    }

    pub fn gacha_manifest_for_app_id(&self, app_id: &QString) -> QString {
        let aid = app_id.to_string();
        let Some((manifest, edition_id, _voices)) =
            omikuji_core::gachas::strategies::find_for_app_id(&aid)
        else {
            return QString::default();
        };
        QString::from(&format!(
            r#"{{"manifest_id":"{}","edition_id":"{}"}}"#,
            manifest.id, edition_id
        ))
    }

    pub fn gacha_posters(&self) -> QString {
        let manifests = omikuji_core::gachas::manifest::load_all();
        let mut map = serde_json::Map::new();
        for m in &manifests {
            let url = omikuji_core::gachas::strategies::resolve_poster(m);
            map.insert(m.id.clone(), serde_json::Value::String(url));
        }
        QString::from(&serde_json::Value::Object(map).to_string())
    }

    pub fn gacha_resolve_poster(&self, manifest_id: &QString) -> QString {
        let id = manifest_id.to_string();
        let Some(m) = omikuji_core::gachas::manifest::find(&id) else {
            return QString::default();
        };
        QString::from(&omikuji_core::gachas::strategies::resolve_poster(&m))
    }

    pub fn fetch_gacha_install_size(
        self: Pin<&mut Self>,
        request_id: &QString,
        manifest_id: &QString,
        edition_id: &QString,
        voices_csv: &QString,
    ) {
        let rid = request_id.to_string();
        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let voices_str = voices_csv.to_string();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[gacha_size] tokio: {}", e);
                    omikuji_core::install_sizes::push(
                        omikuji_core::install_sizes::InstallSizeResult {
                            request_id: rid,
                            download_bytes: 0,
                            install_bytes: 0,
                            error: format!("runtime: {}", e),
                        },
                    );
                    return;
                }
            };

            let pushed = rt.block_on(async move {
                let Some(manifest) = omikuji_core::gachas::manifest::find(&mid) else {
                    return omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid.clone(),
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("unknown manifest: {}", mid),
                    };
                };
                let voices: Vec<String> = voices_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                match omikuji_core::gachas::strategies::fetch_install_size(&manifest, &eid, &voices).await {
                    Ok(sz) => omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: sz.download_bytes,
                        install_bytes: sz.install_bytes,
                        error: String::new(),
                    },
                    Err(e) => {
                        eprintln!("[gacha_size] {}: {}", mid, e);
                        omikuji_core::install_sizes::InstallSizeResult {
                            request_id: rid,
                            download_bytes: 0,
                            install_bytes: 0,
                            error: e.to_string(),
                        }
                    }
                }
            });

            omikuji_core::install_sizes::push(pushed);
        });
    }

    pub fn gacha_check_existing_install(
        &self,
        manifest_id: &QString,
        edition_id: &QString,
        install_path: &QString,
        temp_path: &QString,
    ) -> QString {
        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let path_s = install_path.to_string();
        let temp_s = temp_path.to_string();
        if path_s.trim().is_empty() {
            return QString::from(r#"{"bytes":0,"segments":0,"has_install":false}"#);
        }
        let Some(manifest) = omikuji_core::gachas::manifest::find(&mid) else {
            return QString::from(r#"{"bytes":0,"segments":0,"has_install":false}"#);
        };
        let install = std::path::PathBuf::from(path_s.trim());
        let temp = if temp_s.trim().is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(temp_s.trim()))
        };
        let info = omikuji_core::gachas::strategies::inspect_existing(
            &manifest,
            &eid,
            &install,
            temp.as_deref(),
        );
        let version_json = match &info.installed_version {
            Some(v) => format!(r#""{}""#, v.replace('"', "")),
            None => "null".to_string(),
        };
        QString::from(&format!(
            r#"{{"bytes":{},"segments":{},"has_install":{},"installed_version":{}}}"#,
            info.scratch_bytes, info.segments, info.has_install, version_json
        ))
    }

    pub fn gacha_import_after_install(
        mut self: Pin<&mut Self>,
        manifest_id: &QString,
        edition_id: &QString,
        display_name: &QString,
        install_path: &QString,
        runner_version: &QString,
        prefix_path: &QString,
    ) -> QString {
        use omikuji_core::library::{
            default_color, GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, SourceConfig,
            SystemConfig, WineConfig,
        };

        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let install_s = install_path.to_string();
        let display_s = display_name.to_string();
        let prefix_s = prefix_path.to_string();
        let runner_s = runner_version.to_string();

        let Some(manifest) = omikuji_core::gachas::manifest::find(&mid) else {
            eprintln!("[gacha_import] unknown manifest: {}", mid);
            return QString::default();
        };
        let Some(edition) = manifest.editions.iter().find(|e| e.id == eid) else {
            eprintln!("[gacha_import] unknown edition '{}' for '{}'", eid, mid);
            return QString::default();
        };
        let app_id = omikuji_core::gachas::strategies::build_app_id(&manifest, &eid, &[]);

        if self.library.game.iter().any(|g| {
            g.source.kind == "gacha" && g.source.app_id == app_id
        }) {
            eprintln!("[gacha_import] already in library: {}", app_id);
            return QString::default();
        }

        let exe = std::path::Path::new(&install_s).join(&edition.exe_name);
        let category = if manifest.category.is_empty() {
            "Gacha".to_string()
        } else {
            manifest.category.clone()
        };
        let game_id = omikuji_core::library::generate_id();

        let mut game = Game {
            metadata: Metadata {
                id: game_id.clone(),
                name: display_s.clone(),
                sort_name: String::new(),
                slug: String::new(),
                exe,
                color: default_color(),
                playtime: 0.0,
                last_played: String::new(),
                banner: String::new(),
                coverart: String::new(),
                icon: String::new(),
                favourite: false,
                categories: vec![category],
            },
            source: SourceConfig {
                kind: "gacha".to_string(),
                app_id: app_id.clone(),
                patch: manifest.launch_patch.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: "wine".to_string(),
            },
            wine: WineConfig {
                version: runner_s,
                prefix: prefix_s,
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("[gacha_import] failed to save: {}", e);
            return QString::default();
        }

        let install_path_buf = std::path::PathBuf::from(&install_s);
        if let Some(version) = omikuji_core::gachas::strategies::read_install_version(
            &manifest,
            &edition.id,
            &install_path_buf,
        ) {
            omikuji_core::gachas::state::write_installed_version(
                &manifest.publisher_slug,
                &manifest.game_slug,
                &edition.id,
                &version,
            );
            let dotversion = install_path_buf.join(".version");
            if !dotversion.exists() {
                let _ = std::fs::write(&dotversion, &version);
            }
            eprintln!(
                "[gacha_import] detected version {} for {}/{} {}",
                version, manifest.publisher_slug, manifest.game_slug, edition.id
            );
        } else {
            eprintln!(
                "[gacha_import] coulndt detect version on disk for {}/{} {}, update check skipped until next install",
                manifest.publisher_slug, manifest.game_slug, edition.id
            );
        }

        let id_for_media = game.metadata.id.clone();
        let manifest_for_media = manifest.clone();
        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let id_for_refresh = id_for_media.clone();
            omikuji_core::gachas::art::fetch_into_library_cache(
                &manifest_for_media,
                &id_for_media,
                |_| {
                    let id_inner = id_for_refresh.clone();
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut super::qobject::GameModel>| {
                        let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                            return;
                        };
                        let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
                        let roles = cxx_qt_lib::QList::<i32>::default();
                        obj.as_mut().data_changed(&idx, &idx, &roles);
                    });
                },
            );
        });

        self.as_mut().begin_insert_rows(&QModelIndex::default(), row, row);
        self.as_mut().rust_mut().get_mut().library.game.push(game);
        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        eprintln!(
            "[gacha_import] imported '{}' ({}) as id '{}'",
            display_s, app_id, game_id
        );
        QString::from(&game_id)
    }
}
