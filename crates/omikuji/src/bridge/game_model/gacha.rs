use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use omikuji_core::defaults::Defaults;
use omikuji_core::library::{Game, Library, SourceKind, generate_id};
use omikuji_core::media::MediaSlot;
use omikuji_core::{components, gacha, install_sizes, notifications};

use crate::bridge::{csv_ids, expand_path};

impl super::qobject::GameModel {
    pub fn list_gachas(&self) -> QString {
        let manifests = gacha::manifest::load_all();
        match serde_json::to_string(&manifests) {
            Ok(s) => QString::from(&s),
            Err(e) => {
                tracing::error!("serialize failed: {}", e);
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
                    tracing::error!("couldn't build runtime: {}", e);
                    return;
                }
            };
            let fetched = match rt.block_on(gacha::remote::ensure_all_fetched()) {
                Ok(n) => n,
                Err(e) => {
                    tracing::error!("gacha manifest fetch failed: {}", e);
                    notifications::warning(
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
        match gacha::manifest::find(&id) {
            Some(m) => match serde_json::to_string(&m) {
                Ok(s) => QString::from(&s),
                Err(e) => {
                    tracing::error!("serialize failed: {}", e);
                    QString::default()
                }
            },
            None => QString::default(),
        }
    }

    pub fn gacha_manifest_for_app_id(&self, app_id: &QString) -> QString {
        let aid = app_id.to_string();
        let Some((manifest, edition_id, _voices)) = gacha::strategies::find_for_app_id(&aid) else {
            return QString::default();
        };
        QString::from(&format!(
            r#"{{"manifest_id":"{}","edition_id":"{}"}}"#,
            manifest.id, edition_id
        ))
    }

    pub fn gacha_posters(&self) -> QString {
        let manifests = gacha::manifest::load_all();
        let mut map = serde_json::Map::new();
        for m in &manifests {
            let url = gacha::strategies::resolve_poster(m);
            map.insert(m.id.clone(), serde_json::Value::String(url));
        }
        QString::from(&serde_json::Value::Object(map).to_string())
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

        install_sizes::spawn_fetch(rid, move || async move {
            let manifest =
                gacha::manifest::find(&mid).ok_or_else(|| format!("unknown manifest: {}", mid))?;
            let voices: Vec<String> = voices_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            gacha::strategies::fetch_install_size(&manifest, &eid, &voices)
                .await
                .map(|s| (s.download_bytes, s.install_bytes))
                .map_err(|e| e.to_string())
        });
    }

    pub fn gacha_detect_edition(&self, manifest_id: &QString, install_path: &QString) -> QString {
        let path = expand_path(install_path);
        gacha::manifest::find(&manifest_id.to_string())
            .and_then(|m| gacha::strategies::detect_edition(&m, std::path::Path::new(&path)))
            .map(|e| QString::from(&e))
            .unwrap_or_default()
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
        let path_s = expand_path(install_path);
        let temp_s = expand_path(temp_path);
        if path_s.trim().is_empty() {
            return QString::from(r#"{"bytes":0,"segments":0,"has_install":false}"#);
        }
        let Some(manifest) = gacha::manifest::find(&mid) else {
            return QString::from(r#"{"bytes":0,"segments":0,"has_install":false}"#);
        };
        let install = std::path::PathBuf::from(path_s.trim());
        let temp = if temp_s.trim().is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(temp_s.trim()))
        };
        let info = gacha::strategies::inspect_existing(&manifest, &eid, &install, temp.as_deref());
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
        install_path: &QString,
        runner_version: &QString,
        prefix_path: &QString,
        options_csv: &QString,
    ) -> QString {
        use omikuji_core::library::{
            GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, RunnerType, SourceConfig,
            SystemConfig, WineConfig,
        };

        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let install_s = expand_path(install_path);
        let prefix_s = prefix_path.to_string();
        let runner_s = runner_version.to_string();

        let Some(manifest) = gacha::manifest::find(&mid) else {
            tracing::warn!("unknown manifest: {}", mid);
            return QString::default();
        };
        let Some(edition) = manifest.edition(&eid) else {
            tracing::warn!("unknown edition '{}' for '{}'", eid, mid);
            return QString::default();
        };
        let display_s = manifest.display_name_for(edition);
        let app_id = gacha::strategies::build_app_id(&manifest, &eid, &[]);

        let exe = std::path::Path::new(&install_s).join(&edition.exe_name);

        if self
            .library
            .game
            .iter()
            .any(|g| g.source.kind == SourceKind::Gacha && g.metadata.exe == exe)
        {
            tracing::info!("already in library: {}", exe.display());
            return QString::default();
        }

        let category = if manifest.category.is_empty() {
            "Gacha".to_string()
        } else {
            manifest.category.clone()
        };
        let game_id = generate_id();

        let mut game = Game {
            metadata: Metadata {
                categories: vec![category],
                ..Metadata::new(game_id.clone(), display_s.clone(), exe)
            },
            source: SourceConfig {
                kind: SourceKind::Gacha,
                app_id: app_id.clone(),
                patch: manifest.launch_patch.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: RunnerType::Wine,
            },
            wine: WineConfig {
                version: runner_s,
                prefix: prefix_s,
                ..WineConfig::default()
            },
            launch: LaunchConfig {
                env: manifest.env.clone(),
                ..LaunchConfig::default()
            },
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        let accepted = csv_ids(options_csv);
        let companion = manifest.apply_options(&accepted, &mut game.launch).cloned();
        game.seed_from_defaults(&Defaults::load());

        if let Err(e) = Library::save_game_static(&game) {
            tracing::error!("failed to save: {}", e);
            return QString::default();
        }

        let tools = components::gacha_tools(manifest.strategy_for(edition));
        if !tools.is_empty() {
            tokio::spawn(async move {
                let _ = components::ensure(&tools).await;
            });
        }

        if let Some(spec) = companion {
            tokio::spawn(async move {
                match spec.install().await {
                    Ok(path) => tracing::info!("fetched {} to {}", spec.name, path.display()),
                    Err(e) => tracing::error!("couldn't fetch {}: {:#}", spec.name, e),
                }
            });
        }

        let install_path_buf = std::path::PathBuf::from(&install_s);
        if gacha::state::read_installed_version(&manifest.game_slug, &edition.id).is_none() {
            if let Some(version) =
                gacha::strategies::read_install_version(&manifest, &edition.id, &install_path_buf)
            {
                gacha::state::write_installed_version(&manifest.game_slug, &edition.id, &version);
                let dotversion = install_path_buf.join(".version");
                if !dotversion.exists() {
                    let _ = std::fs::write(&dotversion, &version);
                }
                tracing::info!(
                    "detected version {} for {} {}",
                    version,
                    manifest.game_slug,
                    edition.id
                );
            } else {
                tracing::warn!(
                    "couldn't detect version on disk for {} {}, update check skipped until next install",
                    manifest.game_slug,
                    edition.id
                );
            }
        }

        let id_for_media = game.metadata.id.clone();
        let manifest_for_media = manifest.clone();
        let qt_thread = self.as_mut().qt_thread();
        let on_asset = super::media_changed_notifier(qt_thread, id_for_media.clone());
        std::thread::spawn(move || {
            gacha::art::fetch_into_library_cache(
                MediaSlot::Live,
                &manifest_for_media,
                &id_for_media,
                on_asset,
            );
        });

        self.as_mut().insert_game_sorted(game);

        tracing::info!("imported '{}' ({}) as id '{}'", display_s, app_id, game_id);
        QString::from(&game_id)
    }
}
