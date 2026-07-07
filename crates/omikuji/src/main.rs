mod bridge;
mod cli;
mod log_fmt;
mod single_instance;

use cxx_qt_lib::{QQmlApplicationEngine, QUrl};
use std::ffi::CString;

unsafe extern "C" {
    fn omikuji_app_init();
    fn omikuji_app_exec() -> std::os::raw::c_int;
    fn omikuji_set_window_icon(path: *const std::os::raw::c_char);
    fn omikuji_set_desktop_file_name(name: *const std::os::raw::c_char);
    fn omikuji_capture_default_font();
    fn omikuji_set_app_font(family: *const std::os::raw::c_char);
    fn omikuji_install_translator(lang: *const std::os::raw::c_char);
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

    let qml_root = match &action {
        cli::CliAction::Exit(code) => std::process::exit(*code),
        cli::CliAction::Gui => "qrc:/qt/qml/omikuji/qml/Main.qml",
        cli::CliAction::Console => "qrc:/qt/qml/omikuji/qml/ConsoleMode.qml",
        cli::CliAction::RunExe(exe) => {
            unsafe { std::env::set_var("OMIKUJI_RUN_EXE", exe) };
            "qrc:/qt/qml/omikuji/qml/RunExe.qml"
        }
    };

    if !matches!(action, cli::CliAction::RunExe(_)) && !single_instance::check().await {
        return;
    }

    let had_style_override = std::env::var_os("QT_STYLE_OVERRIDE").is_some();
    unsafe {
        if !had_style_override {
            std::env::set_var("QT_STYLE_OVERRIDE", "Fusion");
        }
        omikuji_app_init();
        if !had_style_override {
            std::env::remove_var("QT_STYLE_OVERRIDE");
        }
    }

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

    unsafe { omikuji_capture_default_font(); }
    if !ui.theme.follow_system_font && !ui.theme.font_family.is_empty()
        && let Ok(family) = CString::new(ui.theme.font_family) {
            unsafe { omikuji_set_app_font(family.as_ptr()) };
        }

    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(qml_root));
    }

    unsafe { omikuji_app_exec(); }
}
