use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use omikuji_core::defaults::Defaults;
use omikuji_core::gacha::controls;
use omikuji_core::library::{Game, Library, SourceKind, generate_id};
use omikuji_core::media::MediaSlot;
use omikuji_core::{anyhow, background, components, gacha, install_sizes, notifications};

use super::{Draft, args_to_text};
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
        let Some((manifest, edition_id)) = gacha::strategies::find_for_app_id(&aid) else {
            return QString::default();
        };
        QString::from(&format!(
            r#"{{"manifest_id":"{}","edition_id":"{}"}}"#,
            manifest.id, edition_id
        ))
    }

    pub fn gacha_packs(&self, game_id: &QString) -> QString {
        let gid = game_id.to_string();
        let Some(game) = self.library.game(&gid) else {
            return QString::default();
        };
        QString::from(
            &serde_json::json!({
                "kind": gacha::strategies::pack_kind(game),
                "packs": gacha::strategies::packs(game),
            })
            .to_string(),
        )
    }

    pub fn gacha_pack_removal(&self, pack: &QString) -> QString {
        let pack = pack.to_string();
        let Some(draft) = &self.rust().draft else {
            return QString::default();
        };
        let active = is_active_pack(&draft.game, &pack);
        let fallback = active
            .then(|| controls::pack_fallback(&draft.game, &pack))
            .flatten()
            .map(|c| c.label);
        QString::from(&serde_json::json!({ "active": active, "fallback": fallback }).to_string())
    }

    pub fn remove_gacha_pack(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        pack: &QString,
    ) -> QString {
        let gid = game_id.to_string();
        let pack = pack.to_string();
        let saved_active = self
            .library
            .game(&gid)
            .is_some_and(|g| is_active_pack(g, &pack));
        if let Some(draft) = self.as_mut().rust_mut().get_mut().draft.as_mut()
            && draft.game.metadata.id == gid
        {
            if is_active_pack(&draft.game, &pack) {
                let fallback = controls::pack_fallback(&draft.game, &pack).map(|c| c.id);
                if let Err(e) = select_on_draft(draft, controls::PACKS_CONTROL, fallback.as_deref())
                {
                    return QString::from(&format!("{e:#}"));
                }
            }
            if saved_active {
                let launch = draft.game.launch.clone();
                self.as_mut().update_game(&gid, |g| {
                    g.launch.args = launch.args;
                    g.launch.env = launch.env;
                });
            }
        }
        let Some(game) = self.library.game(&gid) else {
            return QString::from("game not found");
        };
        match gacha::strategies::remove_pack(game, &pack) {
            Ok(()) => QString::default(),
            Err(e) => {
                tracing::error!("removing {} from {}: {e:?}", pack, game.metadata.name);
                QString::from(&format!("{e:#}"))
            }
        }
    }

    pub fn fetch_gacha_pack_sizes(mut self: Pin<&mut Self>, game_id: &QString) {
        let gid = game_id.to_string();
        let Some((manifest, edition_id)) = self
            .library
            .game(&gid)
            .and_then(|g| gacha::strategies::find_for_app_id(&g.source.app_id))
        else {
            return;
        };
        let qt_thread = self.as_mut().qt_thread();
        background::spawn(
            move || async move {
                gacha::strategies::pack_sizes(&manifest, &edition_id)
                    .await
                    .map_err(|e| format!("{e:#}"))
            },
            move |result| match result {
                Ok(sizes) => {
                    let payload = serde_json::to_string(&sizes).unwrap_or_default();
                    let _ = qt_thread.queue(move |mut m: Pin<&mut super::qobject::GameModel>| {
                        m.as_mut()
                            .pack_sizes_ready(&QString::from(&gid), &QString::from(&payload));
                    });
                }
                Err(e) => tracing::warn!("pack sizes for {gid}: {e}"),
            },
        );
    }

    pub fn gacha_launch_controls(&self) -> QString {
        let controls = self
            .rust()
            .draft
            .as_ref()
            .map(|d| controls::launch_controls(&d.game))
            .unwrap_or_default();
        QString::from(&serde_json::to_string(&controls).unwrap_or_else(|_| "[]".into()))
    }

    pub fn select_gacha_launch_choice(
        mut self: Pin<&mut Self>,
        control_id: &QString,
        choice_id: &QString,
    ) -> QString {
        let Some(draft) = self.as_mut().rust_mut().get_mut().draft.as_mut() else {
            return QString::from("no game is being edited");
        };
        let choice = choice_id.to_string();
        let choice = Some(choice.as_str()).filter(|c| !c.is_empty());
        if let Err(e) = select_on_draft(draft, &control_id.to_string(), choice) {
            return QString::from(&format!("{e:#}"));
        }
        let id = QString::from(&draft.game.metadata.id);
        self.as_mut().draft_rebased(&id);
        QString::default()
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
        packs_csv: &QString,
    ) {
        let rid = request_id.to_string();
        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let packs = csv_ids(packs_csv);

        install_sizes::spawn_fetch(rid, move || async move {
            let manifest =
                gacha::manifest::find(&mid).ok_or_else(|| format!("unknown manifest: {}", mid))?;
            gacha::strategies::fetch_install_size(&manifest, &eid, &packs)
                .await
                .map(|s| (s.download_bytes, s.install_bytes))
                .map_err(|e| e.to_string())
        });
    }

    pub fn gacha_pack_picker(&self, manifest_id: &QString, edition_id: &QString) -> QString {
        let picker = gacha::manifest::find(&manifest_id.to_string())
            .and_then(|m| gacha::strategies::pack_picker(&m, &edition_id.to_string()));
        QString::from(&serde_json::to_string(&picker).unwrap_or_else(|_| "null".into()))
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
        let json = serde_json::json!({
            "bytes": info.scratch_bytes,
            "segments": info.segments,
            "has_install": info.has_install,
            "installed_version": info.installed_version,
            "installed_packs": info.installed_packs,
        });
        QString::from(&json.to_string())
    }

    pub fn gacha_import_after_install(
        mut self: Pin<&mut Self>,
        manifest_id: &QString,
        edition_id: &QString,
        install_path: &QString,
        runner_version: &QString,
        prefix_path: &QString,
        options_csv: &QString,
        packs_csv: &QString,
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
        let app_id = gacha::strategies::build_app_id(&manifest, &eid);

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
                args: manifest.args.clone(),
                ..LaunchConfig::default()
            },
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        let accepted = csv_ids(options_csv);
        let companion = manifest.apply_options(&accepted, &mut game.launch).cloned();
        if let Err(e) = gacha::strategies::apply_pack_effects(
            &manifest,
            &eid,
            &csv_ids(packs_csv),
            &mut game.launch,
        ) {
            tracing::warn!(
                "{} registered without its pack's launch args: {e:#}",
                display_s
            );
        }
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
                    let _ = fs_err::write(&dotversion, &version);
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

fn is_active_pack(game: &Game, pack: &str) -> bool {
    controls::launch_controls(game)
        .into_iter()
        .find(|c| c.id == controls::PACKS_CONTROL)
        .is_some_and(|c| c.selected.as_deref() == Some(pack))
}

fn select_on_draft(
    draft: &mut Draft,
    control_id: &str,
    choice: Option<&str>,
) -> anyhow::Result<()> {
    let control = controls::launch_controls(&draft.game)
        .into_iter()
        .find(|c| c.id == control_id)
        .ok_or_else(|| anyhow::anyhow!("unknown launch control {control_id}"))?;
    let mut launch = draft.game.launch.clone();
    control.select(&mut launch, choice)?;
    draft.edit("launch.args".into(), args_to_text(&launch.args));
    draft.edit(
        "launch.env".into(),
        serde_json::to_string(&launch.env).unwrap_or_default(),
    );
    Ok(())
}
