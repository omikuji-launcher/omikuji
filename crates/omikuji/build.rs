use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "src/qml_tree.rs"]
mod qml_tree;

const QML_ROOT: &str = "qml";

fn walk_files(dir: &str, exts: &[&str]) -> Vec<PathBuf> {
    qml_tree::walk_files(Path::new(dir), exts).unwrap_or_else(|e| panic!("walk {dir}: {e}"))
}

fn path_string(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn collect_icons() -> (Vec<String>, Vec<String>) {
    let files = walk_files("qml/icons", &["svg", "png"]);
    let names = files
        .iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "svg"))
        .filter_map(|p| p.file_stem()?.to_str())
        .filter(|stem| *stem != "app" && !stem.ends_with("_fill"))
        .map(str::to_owned)
        .collect();
    (files.iter().map(|p| path_string(p)).collect(), names)
}

const SOURCE_TRANSLATION: &str = "omikuji_en";

fn compile_translations() -> Vec<String> {
    let dir = Path::new("i18n");
    let _ = fs::create_dir_all(dir);
    let Ok(entries) = fs::read_dir(dir) else {
        return vec![];
    };

    let sources: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("ts"))
        .filter(|p| p.file_stem().and_then(|s| s.to_str()) != Some(SOURCE_TRANSLATION))
        .collect();

    let lrelease = find_qt_tool(&["lrelease6", "lrelease-qt6", "lrelease"]);
    if lrelease.is_none() && !sources.is_empty() {
        println!(
            "cargo:warning=lrelease not found, building without translations (install qt6-tools)"
        );
    }

    let mut paths: Vec<String> = vec![];
    for ts in sources {
        let qm = ts.with_extension("qm");
        if let Some(lrelease) = &lrelease
            && needs_recompile(&ts, &qm)
        {
            let status = Command::new(lrelease)
                .arg("-silent")
                .arg(&ts)
                .arg("-qm")
                .arg(&qm)
                .status()
                .expect("invoke lrelease");
            if !status.success() {
                panic!("lrelease failed for {}", ts.display());
            }
        }
        if qm.exists() {
            let filename = qm.file_name().unwrap().to_string_lossy().into_owned();
            paths.push(format!("i18n/{filename}"));
        }
    }
    paths.sort();
    paths
}

fn find_qt_tool(names: &[&str]) -> Option<PathBuf> {
    for name in names {
        if let Ok(out) = Command::new("which").arg(name).output()
            && out.status.success()
        {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
        for dir in [
            "/usr/lib/qt6/bin",
            "/usr/lib64/qt6/bin",
            "/usr/lib/x86_64-linux-gnu/qt6/bin",
            "/usr/libexec/qt6",
        ] {
            let p = Path::new(dir).join(name);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

fn find_qsb() -> PathBuf {
    find_qt_tool(&["qsb"]).expect("qsb not found; install qt6-shadertools")
}

fn compile_shaders() -> Vec<String> {
    let mut qsb: Option<PathBuf> = None;
    walk_files(QML_ROOT, &["frag", "vert"])
        .into_iter()
        .map(|src| {
            let dest = PathBuf::from(format!("{}.qsb", src.display()));
            if needs_recompile(&src, &dest) {
                let qsb = qsb.get_or_insert_with(find_qsb);
                let status = Command::new(&*qsb)
                    .arg("--qt6")
                    .arg("-o")
                    .arg(&dest)
                    .arg(&src)
                    .status()
                    .expect("invoke qsb");
                if !status.success() {
                    panic!("qsb failed for {}", src.display());
                }
            }
            path_string(&dest)
        })
        .collect()
}

fn needs_recompile(source: &Path, artifact: &Path) -> bool {
    let Ok(artifact_meta) = fs::metadata(artifact) else {
        return true;
    };
    let Ok(src_meta) = fs::metadata(source) else {
        return true;
    };
    match (artifact_meta.modified(), src_meta.modified()) {
        (Ok(a), Ok(s)) => s > a,
        _ => true,
    }
}

fn write_icon_names(names: &[String]) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let out_path = Path::new(&out_dir).join("icon_names.rs");
    let mut content = String::from("pub const ICON_NAMES: &[&str] = &[\n");
    for n in names {
        content.push_str(&format!("    \"{n}\",\n"));
    }
    content.push_str("];\n");
    fs::write(&out_path, content).expect("write icon_names.rs");
}

fn qt_version() -> String {
    for tool in ["qmake6", "qmake"] {
        if let Ok(out) = Command::new(tool).arg("-query").arg("QT_VERSION").output()
            && out.status.success()
        {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !v.is_empty() {
                return v;
            }
        }
    }
    "unknown".to_string()
}

fn main() {
    let (icon_paths, icon_names) = collect_icons();
    write_icon_names(&icon_names);
    println!("cargo:rerun-if-changed={QML_ROOT}");

    println!("cargo:rustc-env=OMIKUJI_QT_VERSION={}", qt_version());

    let shader_paths = compile_shaders();

    let translation_paths = compile_translations();
    println!("cargo:rerun-if-changed=i18n");

    let mut qrc_paths = icon_paths;
    qrc_paths.extend(shader_paths);
    qrc_paths.extend(translation_paths);
    qrc_paths.extend(walk_files(QML_ROOT, &["js"]).iter().map(|p| path_string(p)));

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let app_settings_bridge = kushi::ObjectBridge::new("AppSettingsBridge")
        .external_data("AppSettings", "AppSettings::load()")
        .threading()
        .prop_at("card_zoom", kushi::Kind::F64, "library.card_zoom")
        .prop_at("card_spacing", kushi::Kind::I32, "library.card_spacing")
        .prop_at("card_elevation", kushi::Kind::Bool, "library.card_elevation")
        .prop_at("unload_store_pages", kushi::Kind::Bool, "library.unload_store_pages")
        .prop_at("show_gachas", kushi::Kind::Bool, "tabs.show_gachas")
        .prop_at("show_epic", kushi::Kind::Bool, "tabs.show_epic")
        .prop_at("show_gog", kushi::Kind::Bool, "tabs.show_gog")
        .prop_at("show_nile", kushi::Kind::Bool, "tabs.show_nile")
        .prop_at("show_steam", kushi::Kind::Bool, "tabs.show_steam")
        .prop_at("nav_width", kushi::Kind::I32, "nav.width")
        .prop_at("nav_collapsed", kushi::Kind::Bool, "nav.collapsed")
        .prop_at("minimize_on_launch", kushi::Kind::Bool, "behavior.minimize_on_launch")
        .prop_at("save_game_logs", kushi::Kind::Bool, "behavior.save_game_logs")
        .prop_at("double_click_launches", kushi::Kind::Bool, "behavior.double_click_launches")
        .prop_at("auto_check_epic_updates_on_launch", kushi::Kind::Bool, "behavior.auto_check_epic_updates_on_launch")
        .prop_at("auto_check_gog_updates_on_launch", kushi::Kind::Bool, "behavior.auto_check_gog_updates_on_launch")
        .prop_at("auto_check_updates_on_boot", kushi::Kind::Bool, "behavior.auto_check_updates_on_boot")
        .prop_at("show_tray_icon", kushi::Kind::Bool, "behavior.show_tray_icon")
        .prop_at("discord_show_launcher", kushi::Kind::Bool, "behavior.discord_show_launcher")
        .prop_at("notify_on_download_complete", kushi::Kind::Bool, "behavior.notify_on_download_complete")
        .prop_at("ignore_steam_runners", kushi::Kind::Bool, "behavior.ignore_steam_runners")
        .prop_at("bandwidth_mb_per_sec", kushi::Kind::F64, "download.bandwidth_mb_per_sec")
        .prop_at("epic_workers", kushi::Kind::I32, "download.epic.workers")
        .prop_at("epic_shared_memory_mb", kushi::Kind::I32, "download.epic.shared_memory_mb")
        .prop_at("gog_workers", kushi::Kind::I32, "download.gog.workers")
        .prop_at("gacha_connections", kushi::Kind::I32, "download.gacha.max_connections")
        .prop_at("gacha_patch_threads", kushi::Kind::I32, "download.gacha.patch_threads")
        .prop_custom_apply("ui_scale", kushi::Kind::F64, "display.scale")
        .prop_at("muted_icons", kushi::Kind::Bool, "display.muted_icons")
        .prop_at("filled_icons", kushi::Kind::Bool, "display.filled_icons")
        .prop_at("progress_style", kushi::Kind::QString, "display.progress_style")
        .prop_at("show_hidden", kushi::Kind::Bool, "display.show_hidden")
        .prop_at("dim_hidden", kushi::Kind::Bool, "display.dim_hidden")
        .prop_at("show_steam_prefixes", kushi::Kind::Bool, "display.show_steam_prefixes")
        .prop_at("highlight_logs", kushi::Kind::Bool, "display.highlight_logs")
        .prop_custom_apply("card_flow", kushi::Kind::QString, "display.card_flow")
        .prop_custom_apply("card_sort", kushi::Kind::QString, "display.card_sort")
        .prop_at("card_style", kushi::Kind::QString, "display.card_style")
        .prop_at("card_play_button", kushi::Kind::Bool, "display.card_play_button")
        .prop_at("console_background", kushi::Kind::QString, "console_mode.background")
        .prop_custom_apply("follow_system_colors", kushi::Kind::Bool, "theme.follow_system_colors")
        .prop_custom_apply("follow_system_font", kushi::Kind::Bool, "theme.follow_system_font")
        .prop_custom_apply("font_family", kushi::Kind::QString, "theme.font_family")
        .prop_at(
            "font_family_mono",
            kushi::Kind::QString,
            "theme.font_family_mono",
        )
        .prop_at(
            "font_family_logs",
            kushi::Kind::QString,
            "theme.font_family_logs",
        )
        .prop_readonly("fill_fields", kushi::Kind::Bool, "theme.fill_fields")
        .prop_at("language", kushi::Kind::QString, "language")
        .prop_at("welcome_seen", kushi::Kind::Bool, "state.welcome_seen")
        .qsignal("theme_changed")
        .json_accessor("categories", "Vec<CategoryEntry>", "categories", "categories_changed")
        .json_accessor("env_sets", "Vec<KvSet>", "env_sets", "env_sets_changed")
        .json_accessor("dll_sets", "Vec<KvSet>", "dll_sets", "dll_sets_changed")
        .json_accessor("log_rules", "Vec<LogRule>", "display.log_rules", "log_rules_changed")
        .json_accessor("dialog_sizes", "BTreeMap<String, [f64; 2]>", "dialog_sizes", "dialog_sizes_changed")
        .json_accessor("font_sizes", "BTreeMap<String, u32>", "theme.fonts", "font_sizes_changed")
        .json_accessor("radius_overrides", "BTreeMap<String, u32>", "theme.radii", "radius_overrides_changed")
        .json_accessor("template_vars", "BTreeMap<String, String>", "template_vars", "template_vars_changed")
        .raw_field_persisted("color_overrides", "BTreeMap<String, String>", "s.theme.colors.clone()", "s.theme.colors = self.color_overrides.clone();")
        .raw_field("watcher", "Option<FileWatcher>", "None")
        .raw_field("suppress_reload_until", "Option<Instant>", "None")
        .custom_invokable("initWatcher", "fn init_watcher(self: Pin<&mut AppSettingsBridge>);")
        .custom_invokable("availableIconsJson", "fn available_icons_json(self: &AppSettingsBridge) -> QString;")
        .custom_invokable("colorOverride", "fn color_override(self: &AppSettingsBridge, token: &QString) -> QString;")
        .custom_invokable("setColorOverride", "fn set_color_override(self: Pin<&mut AppSettingsBridge>, token: &QString, hex: &QString);")
        .custom_invokable("overridesJson", "fn overrides_json(self: &AppSettingsBridge) -> QString;")
        .custom_invokable("availableFontsJson", "fn available_fonts_json(self: &AppSettingsBridge) -> QString;")
        .custom_invokable("availableLanguagesJson", "fn available_languages_json(self: &AppSettingsBridge) -> QString;")
        .reload_hook("reload_extras")
        .write_into(&out_dir);

    let download_model_bridge = kushi::ListModelBridge::new("DownloadModel")
        .file_stem("download_model_bridge")
        .item_type("DownloadEntry")
        .items_name("entries")
        .custom_default()
        .qproperty("count", kushi::Kind::I32)
        .qproperty("active_count", kushi::Kind::I32)
        .qproperty("completed_count", kushi::Kind::I32)
        .qproperty("running_count", kushi::Kind::I32)
        .qproperty("queued_count", kushi::Kind::I32)
        .qproperty("failed_count", kushi::Kind::I32)
        .qproperty("hero_id", kushi::Kind::QString)
        .role("id", kushi::Kind::QString, "id")
        .role("source", kushi::Kind::QString, "source")
        .role("app_id", kushi::Kind::QString, "app_id")
        .role("display_name", kushi::Kind::QString, "display_name")
        .role_fn("banner", "role_banner")
        .role_fn("status", "role_status")
        .role("progress", kushi::Kind::F64, "progress")
        .role_fn("speed", "role_speed")
        .role_fn("bytes_downloaded", "role_bytes_downloaded")
        .role_fn("bytes_total", "role_bytes_total")
        .role_fn("error", "role_error")
        .role_fn("kind", "role_kind")
        .qsignal_raw("fn download_completed(self: Pin<&mut DownloadModel>, id: &QString, source: &QString, app_id: &QString, display_name: &QString, install_path: &QString, prefix_path: &QString, runner_version: &QString, dlcs: &QString, alongside: bool);")
        .qsignal_raw("fn download_failed(self: Pin<&mut DownloadModel>, id: &QString, error: &QString);")
        .qsignal_raw("fn state_changed(self: Pin<&mut DownloadModel>);")
        .custom_invokable_raw("fn enqueue_epic(self: Pin<&mut DownloadModel>, app_id: &QString, display_name: &QString, banner_url: &QString, install_path: &QString, prefix_path: &QString, runner_version: &QString) -> QString;")
        .custom_invokable_raw("fn enqueue_gacha(self: Pin<&mut DownloadModel>, manifest_id: &QString, edition_id: &QString, voices_csv: &QString, display_name: &QString, install_path: &QString, runner_version: &QString, prefix_path: &QString, temp_path: &QString, import_existing: bool, alongside: bool) -> QString;")
        .custom_invokable_raw("fn gacha_supports_import(self: &DownloadModel, manifest_id: &QString) -> bool;")
        .custom_invokable_raw("fn pause(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn resume(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn cancel(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn retry(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn dismiss(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn drain_events(self: Pin<&mut DownloadModel>);")
        .custom_invokable_raw(
            "fn source_state_json(self: &DownloadModel, source: &QString) -> QString;",
        )
        .custom_invokable_raw("fn active_for_game_id(self: &DownloadModel, game_id: &QString) -> QString;")
        .custom_invokable("speedHistoryJson", "fn speed_history_json(self: &DownloadModel) -> QString;")
        .custom_invokable("partialBytes", "fn partial_bytes(self: &DownloadModel, id: &QString) -> QString;")
        .row_ops()
        .write_into(&out_dir);

    let staged_bridges = kushi::stage_files(
        [
            "src/bridge/game_model.rs",
            "src/bridge/library_watcher.rs",
            "src/bridge/log_highlighter.rs",
            "src/bridge/epic_model.rs",
            "src/bridge/gog_model.rs",
            "src/bridge/nile_model.rs",
            "src/bridge/components.rs",
            "src/bridge/migration.rs",
            "src/bridge/ofuda.rs",
            "src/bridge/scripts.rs",
            "src/bridge/archive_manager.rs",
            "src/bridge/defaults.rs",
            "src/bridge/gamepad.rs",
            "src/bridge/tray.rs",
        ],
        &out_dir,
    );

    let hot_reload = std::env::var("OMIKUJI_QML_HOTRELOAD").is_ok_and(|v| !v.is_empty());
    println!("cargo:rerun-if-env-changed=OMIKUJI_QML_HOTRELOAD");

    let (singletons, qml_files): (Vec<_>, Vec<_>) = walk_files(QML_ROOT, &["qml"])
        .into_iter()
        .partition(|p| qml_tree::is_singleton(p));

    let mut qml_module = QmlModule::new("omikuji")
        .qml_files(singletons.into_iter().map(|p| QmlFile::from(p).singleton(true)));
    if !hot_reload {
        qml_module = qml_module.qml_files(qml_files);
    }

    let builder = CxxQtBuilder::new_qml_module(qml_module)
        .qrc_resources(&qrc_paths)
        .files(staged_bridges)
        .file(app_settings_bridge)
        .file(download_model_bridge);

    // link QtSvg, QIcon uses the image plugin system to load SVGs.
    // cxx-qt-build's qt_module("Svg") sets include paths but doesn't always add the shared lib to the runtime link
    // force it withan explicit rustc directive so libQt6Svg.so ends up in the dependency graph.
    // without this, QIcon(path) silently returns an empty icon for .svg files and renders blank
    let builder = builder.qt_module("Svg");
    println!("cargo:rustc-link-lib=Qt6Svg");

    let builder = builder.qt_module("Widgets");
    println!("cargo:rustc-link-lib=Qt6Widgets");

    // QQuickTextDocument in log_highlighter needs Qt6Quick linked explicitly, ci shenanigans smh
    let builder = builder.qt_module("Quick");
    println!("cargo:rustc-link-lib=Qt6Quick");

    let builder = builder.qt_module("Qml");
    println!("cargo:rustc-link-lib=Qt6Qml");

    let builder = builder.qt_module("DBus");
    println!("cargo:rustc-link-lib=Qt6DBus");

    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.flag_if_supported("-Wno-sfinae-incomplete");
            cc.file("src/app_icon.cpp");
            cc.file("src/app_font.cpp");
            cc.file("src/tray_native.cpp");
            cc.file("src/i18n.cpp");
            cc.file("src/hot_reload.cpp");
            cc.file("src/notify.cpp");
            cc.file("src/inhibit.cpp");
        })
    };
    println!("cargo:rerun-if-changed=src/app_icon.cpp");
    println!("cargo:rerun-if-changed=src/app_font.cpp");
    println!("cargo:rerun-if-changed=src/tray_native.cpp");
    println!("cargo:rerun-if-changed=src/i18n.cpp");
    println!("cargo:rerun-if-changed=src/hot_reload.cpp");
    println!("cargo:rerun-if-changed=src/notify.cpp");
    println!("cargo:rerun-if-changed=src/inhibit.cpp");

    builder.build();
}
