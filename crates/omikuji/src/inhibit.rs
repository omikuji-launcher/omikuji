use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{LazyLock, Mutex};

unsafe extern "C" {
    fn omikuji_inhibit(reason: *const std::os::raw::c_char) -> u32;
    fn omikuji_uninhibit(cookie: u32);
}

static HELD: LazyLock<Mutex<HashMap<String, u32>>> = LazyLock::new(Default::default);

pub fn acquire(game_id: &str, reason: &str) {
    let Ok(reason) = CString::new(reason) else {
        return;
    };
    let Ok(mut held) = HELD.lock() else {
        return;
    };
    if held.contains_key(game_id) {
        return;
    }
    let cookie = unsafe { omikuji_inhibit(reason.as_ptr()) };
    if cookie != 0 {
        held.insert(game_id.to_string(), cookie);
    }
}

pub fn release(game_id: &str) {
    let Ok(mut held) = HELD.lock() else {
        return;
    };
    if let Some(cookie) = held.remove(game_id) {
        unsafe { omikuji_uninhibit(cookie) };
    }
}
