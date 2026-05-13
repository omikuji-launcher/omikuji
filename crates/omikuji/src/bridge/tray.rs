use std::ffi::CString;
use std::pin::Pin;
use std::sync::Mutex;
use std::sync::OnceLock;

unsafe extern "C" {
    fn omikuji_app_set_quit_on_last_window_closed(v: bool);
    fn omikuji_app_quit();
    fn omikuji_tray_set_icon(path: *const std::os::raw::c_char);
    fn omikuji_tray_set_enabled(enabled: bool);
    fn omikuji_tray_set_recent(json: *const u8, len: usize);
}

#[derive(Debug, Clone)]
enum TrayEvent {
    Show,
    Quit,
    Toggle,
    Game(String),
}

fn queue() -> &'static Mutex<Vec<TrayEvent>> {
    static Q: OnceLock<Mutex<Vec<TrayEvent>>> = OnceLock::new();
    Q.get_or_init(|| Mutex::new(Vec::new()))
}

fn push(ev: TrayEvent) {
    if let Ok(mut q) = queue().lock() {
        q.push(ev);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn omikuji_tray_event_show() { push(TrayEvent::Show); }

#[unsafe(no_mangle)]
pub extern "C" fn omikuji_tray_event_quit() { push(TrayEvent::Quit); }

#[unsafe(no_mangle)]
pub extern "C" fn omikuji_tray_event_toggle() { push(TrayEvent::Toggle); }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn omikuji_tray_event_game(id: *const std::os::raw::c_char, len: usize) {
    if id.is_null() || len == 0 { return; }
    let bytes = unsafe { std::slice::from_raw_parts(id as *const u8, len) };
    if let Ok(s) = std::str::from_utf8(bytes) {
        push(TrayEvent::Game(s.to_string()));
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
        #[cxx_name = "setEnabled"]
        fn set_enabled(self: Pin<&mut TrayBridge>, enabled: bool);

        #[qinvokable]
        #[cxx_name = "setRecentGames"]
        fn set_recent_games(self: Pin<&mut TrayBridge>, json: &QString);

        #[qinvokable]
        #[cxx_name = "setIcon"]
        fn set_icon(self: Pin<&mut TrayBridge>, path: &QString);

        #[qinvokable]
        #[cxx_name = "drainEvents"]
        fn drain_events(self: Pin<&mut TrayBridge>);

        #[qinvokable]
        #[cxx_name = "quitApp"]
        fn quit_app(self: &TrayBridge);
    }
}

#[derive(Default)]
pub struct TrayBridgeRust {}

impl qobject::TrayBridge {
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

    fn drain_events(mut self: Pin<&mut Self>) {
        let events: Vec<TrayEvent> = {
            match queue().lock() {
                Ok(mut q) => std::mem::take(&mut *q),
                Err(_) => return,
            }
        };
        for ev in events {
            match ev {
                TrayEvent::Show | TrayEvent::Toggle => {
                    if matches!(ev, TrayEvent::Toggle) {
                        self.as_mut().toggle_window_requested();
                    } else {
                        self.as_mut().show_window_requested();
                    }
                }
                TrayEvent::Quit => self.as_mut().quit_requested(),
                TrayEvent::Game(id) => {
                    let qid = cxx_qt_lib::QString::from(&id);
                    self.as_mut().launch_game_requested(&qid);
                }
            }
        }
    }

    fn quit_app(&self) {
        unsafe { omikuji_app_quit(); }
    }
}
