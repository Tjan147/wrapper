extern crate libc;

use libc::c_char;

use std::ffi::{CStr, CString};
use std::path::PathBuf;

use storage_proofs::hasher::PedersenHasher;
use super::{error, vanilla};

#[no_mangle]
pub extern "C" fn release(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return
        }
        CString::from_raw(s)
    };
}

fn into_path(input: *const c_char) -> error::Result<PathBuf> {
    let raw_buf = unsafe { CStr::from_ptr(input).to_bytes() };
    let path = String::from_utf8(raw_buf.to_vec())?;

    Ok(PathBuf::from(path))
}

fn into_cstring(e: error::Error) -> CString {
    let e_data = format!("{:?}", e);
    CString::new(e_data).expect("into_cstring: CString::new crash")
}

// #[no_mangle]
// pub extern "C" fn setup(src_file: *const c_char, cache_dir: *const c_char) -> *mut c_char {
//     let src_path = match into_path(src_file) {
//         Err(e) => {
//             return into_cstring(e).into_raw()
//         },
//         Ok(v) => v,
//     };

//     let cache_path = match into_path(cache_dir) {
//         Err(e) => {
//             return into_cstring(e).into_raw()
//         },
//         Ok(v) => v,
//     };

//     // TODO: add path validation logic here

//     // and the main hasher used in the filecoin benchy demo is pedersen
//     match porep::setup_inner::<PedersenHasher>(src_path.as_path(), cache_path.as_path()) {
//         Err(e) => {
//             return into_cstring(e).into_raw()
//         },
//         Ok(_) => {
//             let res = CString::new("0").expect("setup: CString::new crash");
//             res.into_raw()
//         }
//     }
// }