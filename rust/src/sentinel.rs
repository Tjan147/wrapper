extern crate libc;
use std::ffi::CStr;

// FFI API 
// use this simple call as an sentinel for rust-ffi/cgo interaction test
#[no_mangle]
pub extern "C" fn sentinel(name: *const libc::c_char) {
    let buf_name = unsafe { CStr::from_ptr(name).to_bytes() };
    let str_name = String::from_utf8(buf_name.to_vec()).unwrap();
    println!("Hello {}", str_name);
}