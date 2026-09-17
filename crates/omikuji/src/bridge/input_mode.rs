use std::ffi::c_void;
use std::pin::Pin;

use cxx_qt::{CxxQtThread, Threading};

#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, keyboard)]
        type InputModeBridge = super::InputModeBridgeRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        fn start(self: Pin<&mut InputModeBridge>);
    }

    impl cxx_qt::Threading for InputModeBridge {}
}

type ModeCallback = extern "C" fn(*mut c_void, bool);

unsafe extern "C" {
    fn omikuji_watch_input_mode(ctx: *mut c_void, callback: ModeCallback);
}

#[derive(Default)]
pub struct InputModeBridgeRust {
    keyboard: bool,
}

extern "C" fn on_mode_changed(ctx: *mut c_void, keyboard: bool) {
    let thread = unsafe { &*(ctx as *const CxxQtThread<qobject::InputModeBridge>) };
    let _ = thread.queue(move |obj: Pin<&mut qobject::InputModeBridge>| {
        obj.set_keyboard(keyboard);
    });
}

impl qobject::InputModeBridge {
    fn start(self: Pin<&mut Self>) {
        let ctx = Box::into_raw(Box::new(self.qt_thread())) as *mut c_void;
        unsafe { omikuji_watch_input_mode(ctx, on_mode_changed) };
    }
}
