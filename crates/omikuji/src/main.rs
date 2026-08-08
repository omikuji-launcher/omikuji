mod bridge;
mod cli;
mod log_fmt;
mod single_instance;

use cxx_qt_lib::{QQmlApplicationEngine, QUrl};
use std::ffi::{c_void, CString};
use std::path::PathBuf;

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

fn hot_reload_dir() -> Option<PathBuf> {
    match std::env::var("OMIKUJI_QML_HOTRELOAD") {
        Ok(val) if !val.is_empty() => Some(match val.as_str() {
            "1" | "true" => PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            path => PathBuf::from(path),
        }),
        _ => None,
    }
}

fn qml_root_url(rel: &str) -> String {
    match hot_reload_dir() {
        Some(dir) => format!("file://{}", dir.join(rel).display()),
        None => format!("qrc:/qt/qml/omikuji/{rel}"),
    }
}

#[tokio::main]
async fn main() {
    unsafe { std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Basic") };

    tracing_subscriber::fmt()
        .event_format(log_fmt::ShortTarget)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

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

    let qml_root = qml_root_url(qml_rel);
    if qml_root.starts_with("file://") {
        eprintln!("omikuji: qml hot-reload active, loading from {qml_root}");
    }

    if !matches!(action, cli::CliAction::RunExe(_)) && !single_instance::check().await {
        return;
    }

    unsafe { omikuji_app_init() };

    let ui = omikuji_core::ui_settings::UiSettings::load();

    if let Ok(lang) = CString::new(ui.language) {
        unsafe { omikuji_install_translator(lang.as_ptr()) };
    }

    if let Ok(name) = CString::new("omikuji") {
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
        engine.as_mut().load(&QUrl::from(qml_root.as_str()));

        if let Some(dir) = hot_reload_dir() {
            let engine_ptr = unsafe {
                engine.as_mut().get_unchecked_mut() as *mut QQmlApplicationEngine as *mut c_void
            };
            if let (Ok(qml_dir), Ok(root)) = (
                CString::new(dir.join("qml").to_string_lossy().as_bytes()),
                CString::new(qml_root.as_bytes()),
            ) {
                unsafe { omikuji_start_qml_watcher(engine_ptr, qml_dir.as_ptr(), root.as_ptr()) };
            }
        }
    }

    unsafe {
        omikuji_app_exec();
    }
}
