mod bridge;
mod cli;

use cxx_qt_lib::{QQmlApplicationEngine, QUrl};
use std::ffi::CString;

unsafe extern "C" {
    fn omikuji_app_init();
    fn omikuji_app_exec() -> std::os::raw::c_int;
    fn omikuji_set_window_icon(path: *const std::os::raw::c_char);
    fn omikuji_set_desktop_file_name(name: *const std::os::raw::c_char);
    fn omikuji_capture_default_font();
    fn omikuji_set_app_font(family: *const std::os::raw::c_char);
}

#[tokio::main]
async fn main() {
    let qml_root = match cli::dispatch() {
        cli::CliAction::Exit(code) => std::process::exit(code),
        cli::CliAction::Gui => "qrc:/qt/qml/omikuji/qml/Main.qml",
        cli::CliAction::Console => "qrc:/qt/qml/omikuji/qml/ConsoleMode.qml",
    };

    unsafe { omikuji_app_init(); }

    if let Ok(name) = CString::new("omikuji") {
        unsafe { omikuji_set_desktop_file_name(name.as_ptr()) };
    }

    if let Ok(path) = CString::new(":/qt/qml/omikuji/qml/icons/app.png") {
        unsafe { omikuji_set_window_icon(path.as_ptr()) };
    }

    unsafe { omikuji_capture_default_font(); }
    {
        let ui = omikuji_core::ui_settings::UiSettings::load();
        if !ui.theme.follow_system_font && !ui.theme.font_family.is_empty() {
            if let Ok(family) = CString::new(ui.theme.font_family) {
                unsafe { omikuji_set_app_font(family.as_ptr()) };
            }
        }
    }

    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(qml_root));
    }

    unsafe { omikuji_app_exec(); }
}
