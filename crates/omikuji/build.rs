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
        if ext == "svg" && stem != "app" {
            names.push(stem);
        }
    }
    paths.sort();
    names.sort();
    (paths, names)
}

fn collect_translations() -> Vec<String> {
    let dir = Path::new("i18n");
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
                "qml/components/dialogs/GameLogsWindow.qml",
                "qml/components/dialogs/MigrationDialog.qml",
                "qml/components/dialogs/SetsDialog.qml",
                "qml/components/dialogs/UpdateAvailableDialog.qml",
                // downloads
                "qml/components/downloads/ComponentRow.qml",
                "qml/components/downloads/DownloadRow.qml",
                "qml/components/downloads/DownloadsPage.qml",
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
                "qml/components/popups/PopupSurface.qml",
                "qml/components/cards/StatusBadge.qml",
                "qml/components/cards/StoreCardAction.qml",
                "qml/components/primitives/Squircle.qml",
                "qml/components/primitives/SvgIcon.qml",
                "qml/components/primitives/ThinScrollBar.qml",
                "qml/components/popups/ToastManager.qml",
                "qml/components/popups/Tooltip.qml",
                "qml/components/primitives/WavyProgressBar.qml",
            ])
    )
    .qrc_resources(&qrc_paths)
    .files([
        "src/bridge/game_model.rs",
        "src/bridge/library_watcher.rs",
        "src/bridge/epic_model.rs",
        "src/bridge/gog_model.rs",
        "src/bridge/download_model.rs",
        "src/bridge/ui_settings.rs",
        "src/bridge/components.rs",
        "src/bridge/migration.rs",
        "src/bridge/ofuda.rs",
        "src/bridge/archive_manager.rs",
        "src/bridge/defaults.rs",
        "src/bridge/gamepad.rs",
        "src/bridge/tray.rs",
    ])
    ;

    // link QtSvg, QIcon uses the image plugin system to load SVGs.
    // cxx-qt-build's qt_module("Svg") sets include paths but doesn't always add the shared lib to the runtime link
    // force it withan explicit rustc directive so libQt6Svg.so ends up in the dependency graph. 
    // without this, QIcon(path) silently returns an empty icon for .svg files and renders blank
    let builder = builder.qt_module("Svg");
    println!("cargo:rustc-link-lib=Qt6Svg");

    let builder = builder.qt_module("Widgets");
    println!("cargo:rustc-link-lib=Qt6Widgets");

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
