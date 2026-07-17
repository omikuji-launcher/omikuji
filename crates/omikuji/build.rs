use cxx_qt_build::{CxxQtBuilder, QmlModule};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn collect_icons() -> (Vec<String>, Vec<String>) {
    let dir = Path::new("qml/icons");
    let mut paths: Vec<String> = vec![];
    let mut names: Vec<String> = vec![];
    for entry in fs::read_dir(dir).expect("qml/icons must exist") {
        let entry = entry.expect("read qml/icons entry");
        let p = entry.path();
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(ext, "svg" | "png") {
            continue;
        }
        let filename = p.file_name().unwrap().to_string_lossy().into_owned();
        let stem = filename.strip_suffix(&format!(".{ext}")).unwrap().to_string();
        paths.push(format!("qml/icons/{filename}"));
        if ext == "svg" && stem != "app" && !stem.ends_with("_fill") {
            names.push(stem);
        }
    }
    paths.sort();
    names.sort();
    (paths, names)
}

fn collect_translations() -> Vec<String> {
    let dir = Path::new("i18n");
    let _ = fs::create_dir_all(dir);
    let Ok(entries) = fs::read_dir(dir) else {
        return vec![];
    };
    let mut paths: Vec<String> = vec![];
    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("qm") {
            let filename = p.file_name().unwrap().to_string_lossy().into_owned();
            paths.push(format!("i18n/{filename}"));
        }
    }
    paths.sort();
    paths
}

fn find_qsb() -> PathBuf {
    if let Ok(out) = Command::new("which").arg("qsb").output()
        && out.status.success()
    {
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }
    for candidate in [
        "/usr/lib/qt6/bin/qsb",
        "/usr/lib64/qt6/bin/qsb",
        "/usr/lib/x86_64-linux-gnu/qt6/bin/qsb",
        "/usr/libexec/qt6/qsb",
    ] {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return p;
        }
    }
    panic!("qsb not found; install qt6-shadertools");
}

fn compile_shaders() -> Vec<String> {
    let dir = Path::new("qml/components/consolemode/shaders");
    if !dir.exists() {
        return vec![];
    }
    let mut qsb: Option<PathBuf> = None;
    let mut out_paths: Vec<String> = vec![];
    for entry in fs::read_dir(dir).expect("read shader dir") {
        let entry = entry.expect("read shader entry");
        let p = entry.path();
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(ext, "frag" | "vert") {
            continue;
        }
        let filename = p.file_name().unwrap().to_string_lossy().into_owned();
        let qsb_filename = format!("{filename}.qsb");
        let qsb_dest = dir.join(&qsb_filename);

        if needs_recompile(&p, &qsb_dest) {
            let qsb = qsb.get_or_insert_with(find_qsb);
            let status = Command::new(&*qsb)
                .arg("--qt6")
                .arg("-o")
                .arg(&qsb_dest)
                .arg(&p)
                .status()
                .expect("invoke qsb");
            if !status.success() {
                panic!("qsb failed for {filename}");
            }
        }

        out_paths.push(format!(
            "qml/components/consolemode/shaders/{qsb_filename}"
        ));
    }
    out_paths.sort();
    out_paths
}

fn needs_recompile(source: &Path, qsb: &Path) -> bool {
    let Ok(qsb_meta) = fs::metadata(qsb) else { return true };
    let Ok(src_meta) = fs::metadata(source) else { return true };
    match (qsb_meta.modified(), src_meta.modified()) {
        (Ok(q), Ok(s)) => s > q,
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
    println!("cargo:rerun-if-changed=qml/icons");

    println!("cargo:rustc-env=OMIKUJI_QT_VERSION={}", qt_version());

    let shader_paths = compile_shaders();
    println!("cargo:rerun-if-changed=qml/components/consolemode/shaders");

    let translation_paths = collect_translations();
    println!("cargo:rerun-if-changed=i18n");

    let mut qrc_paths = icon_paths;
    qrc_paths.extend(shader_paths);
    qrc_paths.extend(translation_paths);
    qrc_paths.push("qml/components/lib/RunnerGrouping.js".to_string());
    qrc_paths.push("qml/components/lib/Format.js".to_string());

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ui_settings_bridge = kushi::ObjectBridge::new("UiSettingsBridge")
        .external_data("UiSettings", "UiSettings::load()")
        .threading()
        .prop_at("card_zoom", kushi::Kind::F64, "library.card_zoom")
        .prop_at("card_spacing", kushi::Kind::I32, "library.card_spacing")
        .prop_at("card_elevation", kushi::Kind::Bool, "library.card_elevation")
        .prop_at("unload_store_pages", kushi::Kind::Bool, "library.unload_store_pages")
        .prop_at("show_gachas", kushi::Kind::Bool, "tabs.show_gachas")
        .prop_at("show_epic", kushi::Kind::Bool, "tabs.show_epic")
        .prop_at("show_gog", kushi::Kind::Bool, "tabs.show_gog")
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
        .prop_custom_apply("discord_rpc", kushi::Kind::Bool, "behavior.discord_rpc")
        .prop_custom_apply("ui_scale", kushi::Kind::F64, "display.scale")
        .prop_at("muted_icons", kushi::Kind::Bool, "display.muted_icons")
        .prop_at("filled_icons", kushi::Kind::Bool, "display.filled_icons")
        .prop_at("show_hidden", kushi::Kind::Bool, "display.show_hidden")
        .prop_at("dim_hidden", kushi::Kind::Bool, "display.dim_hidden")
        .prop_at("highlight_logs", kushi::Kind::Bool, "display.highlight_logs")
        .prop_custom_apply("card_flow", kushi::Kind::QString, "display.card_flow")
        .prop_custom_apply("card_sort", kushi::Kind::QString, "display.card_sort")
        .prop_at("card_style", kushi::Kind::QString, "display.card_style")
        .prop_at("console_background", kushi::Kind::QString, "console_mode.background")
        .prop_custom_apply("follow_system_colors", kushi::Kind::Bool, "theme.follow_system_colors")
        .prop_custom_apply("follow_system_font", kushi::Kind::Bool, "theme.follow_system_font")
        .prop_custom_apply("font_family", kushi::Kind::QString, "theme.font_family")
        .prop_readonly("fill_fields", kushi::Kind::Bool, "theme.fill_fields")
        .prop_readonly("radius_scale", kushi::Kind::F64, "theme.radius_scale")
        .prop_at("language", kushi::Kind::QString, "language")
        .qsignal("theme_changed")
        .json_accessor("categories", "Vec<CategoryEntry>", "categories", "categories_changed")
        .json_accessor("env_sets", "Vec<KvSet>", "env_sets", "env_sets_changed")
        .json_accessor("dll_sets", "Vec<KvSet>", "dll_sets", "dll_sets_changed")
        .json_accessor("log_rules", "Vec<LogRule>", "display.log_rules", "log_rules_changed")
        .json_accessor("dialog_sizes", "BTreeMap<String, [f64; 2]>", "dialog_sizes", "dialog_sizes_changed")
        .json_accessor("font_sizes", "BTreeMap<String, u32>", "theme.fonts", "font_sizes_changed")
        .json_accessor("template_vars", "BTreeMap<String, String>", "template_vars", "template_vars_changed")
        .raw_field_persisted("color_overrides", "BTreeMap<String, String>", "s.theme.colors.clone()", "s.theme.colors = self.color_overrides.clone();")
        .raw_field("watcher", "Option<FileWatcher>", "None")
        .raw_field("suppress_reload_until", "Option<Instant>", "None")
        .custom_invokable("initWatcher", "fn init_watcher(self: Pin<&mut UiSettingsBridge>);")
        .custom_invokable("availableIconsJson", "fn available_icons_json(self: &UiSettingsBridge) -> QString;")
        .custom_invokable("colorOverride", "fn color_override(self: &UiSettingsBridge, token: &QString) -> QString;")
        .custom_invokable("setColorOverride", "fn set_color_override(self: Pin<&mut UiSettingsBridge>, token: &QString, hex: &QString);")
        .custom_invokable("overridesJson", "fn overrides_json(self: &UiSettingsBridge) -> QString;")
        .custom_invokable("availableFontsJson", "fn available_fonts_json(self: &UiSettingsBridge) -> QString;")
        .custom_invokable("availableLanguagesJson", "fn available_languages_json(self: &UiSettingsBridge) -> QString;")
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
        .qsignal_raw("fn download_completed(self: Pin<&mut DownloadModel>, id: &QString, source: &QString, app_id: &QString, display_name: &QString, install_path: &QString, prefix_path: &QString, runner_version: &QString);")
        .qsignal_raw("fn download_failed(self: Pin<&mut DownloadModel>, id: &QString, error: &QString);")
        .qsignal_raw("fn state_changed(self: Pin<&mut DownloadModel>);")
        .custom_invokable_raw("fn enqueue_epic(self: Pin<&mut DownloadModel>, app_id: &QString, display_name: &QString, banner_url: &QString, install_path: &QString, prefix_path: &QString, runner_version: &QString) -> QString;")
        .custom_invokable_raw("fn enqueue_gacha(self: Pin<&mut DownloadModel>, manifest_id: &QString, edition_id: &QString, voices_csv: &QString, display_name: &QString, install_path: &QString, runner_version: &QString, prefix_path: &QString, temp_path: &QString) -> QString;")
        .custom_invokable_raw("fn pause(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn resume(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn cancel(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn retry(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn dismiss(self: Pin<&mut DownloadModel>, id: &QString);")
        .custom_invokable_raw("fn drain_events(self: Pin<&mut DownloadModel>);")
        .custom_invokable_raw("fn epic_state_json(self: &DownloadModel) -> QString;")
        .custom_invokable_raw("fn gog_state_json(self: &DownloadModel) -> QString;")
        .custom_invokable_raw("fn active_for_app_id(self: &DownloadModel, app_id: &QString) -> QString;")
        .custom_invokable("speedHistoryJson", "fn speed_history_json(self: &DownloadModel) -> QString;")
        .row_ops()
        .write_into(&out_dir);

    let staged_bridges = kushi::stage_files(
        [
            "src/bridge/game_model.rs",
            "src/bridge/library_watcher.rs",
            "src/bridge/log_highlighter.rs",
            "src/bridge/epic_model.rs",
            "src/bridge/gog_model.rs",
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

    // holy fucking shit this is wild actually
    let builder = CxxQtBuilder::new_qml_module(
        QmlModule::new("omikuji")
            .qml_files([
                "qml/Main.qml",
                "qml/ConsoleMode.qml",
                "qml/RunExe.qml",
                // root
                "qml/components/Theme.qml",

                "qml/components/consolemode/ConsoleCard.qml",
                "qml/components/consolemode/ConsoleCardRow.qml",
                "qml/components/consolemode/ConsoleHintBar.qml",
                "qml/components/consolemode/ConsoleOsk.qml",
                "qml/components/consolemode/ConsolePlayButton.qml",
                "qml/components/consolemode/ConsoleSettingsDialog.qml",
                "qml/components/consolemode/ConsoleTopBar.qml",

                "qml/components/categories/CategoriesController.qml",
                // dialogs
                "qml/components/dialogs/ArchiveManageDialog.qml",
                "qml/components/dialogs/ArchiveSourceDialog.qml",
                "qml/components/dialogs/CategoryEditDialog.qml",
                "qml/components/dialogs/ConfirmDialog.qml",
                "qml/components/popups/ContextMenu.qml",
                "qml/components/dialogs/DialogCard.qml",
                "qml/components/dialogs/DefaultsApplyDialog.qml",
                "qml/components/store/EpicInstallDialog.qml",
                "qml/components/store/GachaInstallDialog.qml",
                "qml/components/dialogs/GameCategoriesDialog.qml",
                "qml/components/store/GogInstallDialog.qml",
                "qml/components/dialogs/ErrorDialog.qml",
                "qml/components/dialogs/PrefixCreateDialog.qml",
                "qml/components/dialogs/PrefixDetailDialog.qml",
                "qml/components/dialogs/PrefixPrepDialog.qml",
                "qml/components/dialogs/RunCommandDialog.qml",
                "qml/components/dialogs/TemplateVarsDialog.qml",
                "qml/components/dialogs/FontSizesDialog.qml",
                "qml/components/controls/ExpansionHint.qml",
                "qml/components/dialogs/LogRulesDialog.qml",
                "qml/components/dialogs/GameLogsWindow.qml",
                "qml/components/dialogs/MigrationDialog.qml",
                "qml/components/dialogs/SetsDialog.qml",
                "qml/components/dialogs/ScriptBrowserDialog.qml",
                "qml/components/dialogs/ScriptRunDialog.qml",
                "qml/components/dialogs/UpdateAvailableDialog.qml",
                // downloads
                "qml/components/downloads/BannerThumb.qml",
                "qml/components/downloads/CapsLabel.qml",
                "qml/components/downloads/ComponentRow.qml",
                "qml/components/downloads/DownloadsPage.qml",
                "qml/components/downloads/HeroCard.qml",
                "qml/components/downloads/KindChip.qml",
                "qml/components/downloads/MiniRow.qml",
                // library
                "qml/components/library/FloatingBar.qml",
                "qml/components/library/GameCard.qml",
                "qml/components/library/GameContextMenu.qml",
                "qml/components/library/GameGrid.qml",
                "qml/components/library/GameIcon.qml",
                // navigation
                "qml/components/navigation/NavTabs.qml",
                "qml/components/navigation/Sidebar.qml",
                "qml/components/navigation/SubNavRail.qml",
                "qml/components/navigation/TopBar.qml",
                // pages
                "qml/components/modals/AddGamePage.qml",
                "qml/components/modals/GameSettingsPage.qml",
                "qml/components/modals/GlobalSettingsPage.qml",
                "qml/components/modals/SettingsModal.qml",
                // settings
                "qml/components/settings/ArchiveSourceRow.qml",
                "qml/components/settings/SettingsRow.qml",
                "qml/components/settings/SettingsSection.qml",
                "qml/components/settings/TabEpic.qml",
                "qml/components/settings/TabGameInfo.qml",
                "qml/components/settings/TabGlobalAbout.qml",
                "qml/components/settings/TabGlobalComponents.qml",
                "qml/components/settings/TabGlobalDefaults.qml",
                "qml/components/settings/TabGlobalOfuda.qml",
                "qml/components/settings/TabGlobalPresets.qml",
                "qml/components/settings/TabGlobalTheme.qml",
                "qml/components/settings/TabGlobalUi.qml",
                "qml/components/settings/TabRunnerOptions.qml",
                "qml/components/settings/TabSystem.qml",
                // store
                "qml/components/store/EpicLibrary.qml",
                "qml/components/store/GachaLibrary.qml",
                "qml/components/store/GogLibrary.qml",
                "qml/components/store/EpicController.qml",
                "qml/components/store/GachaController.qml",
                "qml/components/store/GogController.qml",
                "qml/components/store/StorePanel.qml",
                "qml/components/store/SteamLibrary.qml",
                "qml/components/store/StoreLoginOverlay.qml",
                // widgets
                "qml/components/cards/BaseCard.qml",
                "qml/components/cards/CardGrid.qml",
                "qml/components/popups/DisplayOptionsPopup.qml",
                "qml/components/controls/FieldSurface.qml",
                "qml/components/controls/IconButton.qml",
                "qml/components/popups/IconPickerPopup.qml",
                "qml/components/controls/KeyValueTable.qml",
                "qml/components/controls/LabeledSwitch.qml",
                "qml/components/primitives/LoadingDots.qml",
                "qml/components/controls/M3Button.qml",
                "qml/components/controls/M3Dropdown.qml",
                "qml/components/controls/M3FileField.qml",
                "qml/components/controls/M3Slider.qml",
                "qml/components/controls/M3SpinBox.qml",
                "qml/components/controls/M3Switch.qml",
                "qml/components/controls/M3TextField.qml",
                "qml/components/controls/OutputLog.qml",
                "qml/components/controls/PlayButton.qml",
                "qml/components/controls/ResizeGrips.qml",
                "qml/components/controls/ThemedLogHighlighter.qml",
                "qml/components/popups/PopupSurface.qml",
                "qml/components/popups/PopupZoom.qml",
                "qml/components/cards/StatusBadge.qml",
                "qml/components/cards/StoreCardAction.qml",
                "qml/components/primitives/Sparkline.qml",
                "qml/components/primitives/Squircle.qml",
                "qml/components/primitives/SvgIcon.qml",
                "qml/components/primitives/ThinScrollBar.qml",
                "qml/components/popups/ToastManager.qml",
                "qml/components/popups/Tooltip.qml",
                "qml/components/primitives/WavyProgressBar.qml",
            ])
    )
    .qrc_resources(&qrc_paths)
    .files(staged_bridges)
    .file(ui_settings_bridge)
    .file(download_model_bridge)
    ;

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

    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.flag_if_supported("-Wno-sfinae-incomplete");
            cc.file("src/app_icon.cpp");
            cc.file("src/app_font.cpp");
            cc.file("src/tray_native.cpp");
            cc.file("src/i18n.cpp");
        })
    };
    println!("cargo:rerun-if-changed=src/app_icon.cpp");
    println!("cargo:rerun-if-changed=src/app_font.cpp");
    println!("cargo:rerun-if-changed=src/tray_native.cpp");
    println!("cargo:rerun-if-changed=src/i18n.cpp");

    builder.build();
}
