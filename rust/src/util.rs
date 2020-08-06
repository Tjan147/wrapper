extern crate libc;

use libc::c_char;
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn release(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}
