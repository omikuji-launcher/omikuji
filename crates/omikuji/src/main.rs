mod app_log;
mod bridge;
mod cli;
mod hot_reload;
mod inhibit;
mod log_fmt;
mod notify;
mod qml_tree;
mod single_instance;

use cxx_qt_lib::{QQmlApplicationEngine, QString, QUrl};
use std::ffi::{CString, c_void};

unsafe extern "C" {
    fn omikuji_app_init();
    fn omikuji_app_exec() -> std::os::raw::c_int;
    fn omikuji_set_window_icon(path: *const std::os::raw::c_char);
    fn omikuji_set_desktop_file_name(name: *const std::os::raw::c_char);
    fn omikuji_capture_default_font();
    fn omikuji_set_app_font(family: *const std::os::raw::c_char);
    fn omikuji_install_translator(lang: *const std::os::raw::c_char);
    fn omikuji_start_qml_watcher(
        engine: *mut c_void,
        qml_dir: *const std::os::raw::c_char,
        root_url: *const std::os::raw::c_char,
    );
}

#[tokio::main]
async fn main() {
    unsafe { std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Basic") };
    if std::env::var_os("APPIMAGE").is_some() {
        unsafe { std::env::set_var("QT_QPA_PLATFORMTHEME", "xdgdesktopportal") };
    }

    app_log::init();

    let action = cli::dispatch();

    let qml_rel = match &action {
        cli::CliAction::Exit(code) => std::process::exit(*code),
        cli::CliAction::Gui => "qml/Main.qml",
        cli::CliAction::Console => "qml/ConsoleMode.qml",
        cli::CliAction::RunExe(exe) => {
            unsafe { std::env::set_var("OMIKUJI_RUN_EXE", exe) };
            "qml/RunExe.qml"
        }
    };

    let hot_source = hot_reload::source_dir();
    let disk_module = hot_source
        .as_deref()
        .map(|dir| hot_reload::DiskModule::mount(dir).expect("mount the hot reload qml module"));
    let qml_root = match &disk_module {
        Some(module) => {
            let url = module.file_url(qml_rel);
            eprintln!("omikuji: qml hot-reload active, loading from {url}");
            url
        }
        None => format!("qrc:/qt/qml/omikuji/{qml_rel}"),
    };

    if !matches!(action, cli::CliAction::RunExe(_)) {
        if !single_instance::check().await {
            return;
        }
        app_log::start_session();
    }

    unsafe { omikuji_app_init() };

    let ui = omikuji_core::app_settings::AppSettings::load();

    if let Ok(lang) = CString::new(ui.language) {
        unsafe { omikuji_install_translator(lang.as_ptr()) };
    }

    if let Ok(name) = CString::new("io.github.reakjra.omikuji") {
        unsafe { omikuji_set_desktop_file_name(name.as_ptr()) };
    }

    if let Ok(path) = CString::new(":/qt/qml/omikuji/qml/icons/app.png") {
        unsafe { omikuji_set_window_icon(path.as_ptr()) };
    }

    unsafe {
        omikuji_capture_default_font();
    }
    if !ui.theme.follow_system_font
        && !ui.theme.font_family.is_empty()
        && let Ok(family) = CString::new(ui.theme.font_family)
    {
        unsafe { omikuji_set_app_font(family.as_ptr()) };
    }

    let mut engine = QQmlApplicationEngine::new();

    if let Some(mut engine) = engine.as_mut() {
        if let Some(module) = &disk_module {
            engine
                .as_mut()
                .add_import_path(&QString::from(&*module.import_path().to_string_lossy()));
        }

        engine.as_mut().load(&QUrl::from(qml_root.as_str()));

        if let Some(dir) = &hot_source
            && let (Ok(qml_dir), Ok(root)) = (
                CString::new(dir.join("qml").to_string_lossy().as_bytes()),
                CString::new(qml_root.as_bytes()),
            )
        {
            let engine_ptr = unsafe {
                engine.as_mut().get_unchecked_mut() as *mut QQmlApplicationEngine as *mut c_void
            };
            unsafe { omikuji_start_qml_watcher(engine_ptr, qml_dir.as_ptr(), root.as_ptr()) };
        }
    }

    unsafe {
        omikuji_app_exec();
    }
}
