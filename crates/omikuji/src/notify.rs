use std::ffi::CString;

unsafe extern "C" {
    fn omikuji_notify(title: *const std::os::raw::c_char, body: *const std::os::raw::c_char);
}

pub fn send(title: &str, body: &str) {
    let (Ok(title), Ok(body)) = (CString::new(title), CString::new(body)) else {
        return;
    };
    unsafe { omikuji_notify(title.as_ptr(), body.as_ptr()) };
}
