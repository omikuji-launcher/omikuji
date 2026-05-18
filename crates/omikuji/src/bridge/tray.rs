use std::ffi::CString;
use std::pin::Pin;
use std::sync::OnceLock;

unsafe extern "C" {
    fn omikuji_app_set_quit_on_last_window_closed(v: bool);
    fn omikuji_app_quit();
    fn omikuji_tray_set_icon(path: *const std::os::raw::c_char);
    fn omikuji_tray_set_enabled(enabled: bool);
    fn omikuji_tray_set_recent(json: *const u8, len: usize);
}

static TRAY_THREAD: OnceLock<cxx_qt::CxxQtThread<qobject::TrayBridge>> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn omikuji_tray_event_show() {
    if let Some(thread) = TRAY_THREAD.get() {
        thread.queue(|mut qobject| {
            qobject.as_mut().show_window_requested();
        }).ok();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omikuji_tray_event_quit() {
    if let Some(thread) = TRAY_THREAD.get() {
        thread.queue(|mut qobject| {
            qobject.as_mut().quit_requested();
        }).ok();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omikuji_tray_event_toggle() {
    if let Some(thread) = TRAY_THREAD.get() {
        thread.queue(|mut qobject| {
            qobject.as_mut().toggle_window_requested();
        }).ok();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn omikuji_tray_event_game(id: *const std::os::raw::c_char, len: usize) {
    if id.is_null() || len == 0 { return; }
    let bytes = unsafe { std::slice::from_raw_parts(id as *const u8, len) };
    if let Ok(s) = std::str::from_utf8(bytes) {
        let s = s.to_string();
        if let Some(thread) = TRAY_THREAD.get() {
            thread.queue(move |mut qobject| {
                let qid = cxx_qt_lib::QString::from(&s);
                qobject.as_mut().launch_game_requested(&qid);
            }).ok();
        }
    }
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type TrayBridge = super::TrayBridgeRust;
    }

    impl cxx_qt::Threading for TrayBridge {}

    unsafe extern "RustQt" {
        #[qsignal]
        fn show_window_requested(self: Pin<&mut TrayBridge>);

        #[qsignal]
        fn toggle_window_requested(self: Pin<&mut TrayBridge>);

        #[qsignal]
        fn quit_requested(self: Pin<&mut TrayBridge>);

        #[qsignal]
        fn launch_game_requested(self: Pin<&mut TrayBridge>, game_id: &QString);

        #[qinvokable]
        #[cxx_name = "initThread"]
        fn init_thread(self: Pin<&mut TrayBridge>);

        #[qinvokable]
        #[cxx_name = "setEnabled"]
        fn set_enabled(self: Pin<&mut TrayBridge>, enabled: bool);

        #[qinvokable]
        #[cxx_name = "setRecentGames"]
        fn set_recent_games(self: Pin<&mut TrayBridge>, json: &QString);

        #[qinvokable]
        #[cxx_name = "setIcon"]
        fn set_icon(self: Pin<&mut TrayBridge>, path: &QString);

        #[qinvokable]
        #[cxx_name = "quitApp"]
        fn quit_app(self: &TrayBridge);
    }
}

use cxx_qt::Threading;

#[derive(Default)]
pub struct TrayBridgeRust {}

impl qobject::TrayBridge {
    fn init_thread(self: Pin<&mut Self>) {
        let _ = TRAY_THREAD.set(self.qt_thread());
    }

    fn set_enabled(self: Pin<&mut Self>, enabled: bool) {
        unsafe {
            omikuji_app_set_quit_on_last_window_closed(!enabled);
            omikuji_tray_set_enabled(enabled);
        }
    }

    fn set_recent_games(self: Pin<&mut Self>, json: &cxx_qt_lib::QString) {
        let s = json.to_string();
        let bytes = s.as_bytes();
        unsafe { omikuji_tray_set_recent(bytes.as_ptr(), bytes.len()); }
    }

    fn set_icon(self: Pin<&mut Self>, path: &cxx_qt_lib::QString) {
        let s = path.to_string();
        if let Ok(c) = CString::new(s) {
            unsafe { omikuji_tray_set_icon(c.as_ptr()); }
        }
    }

    fn quit_app(&self) {
        unsafe { omikuji_app_quit(); }
    }
}
