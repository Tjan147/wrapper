extern crate libc;

use libc::c_char;

use std::ffi::{CStr, CString};
use std::io;
use std::path::Path;
// WARNING: some OS specified code applied here
use std::os::unix::ffi::OsStrExt;

use storage_proofs::merkle::BinaryMerkleTree;
use storage_proofs::hasher::PedersenHasher;
use super::{error, param, util, vanilla};

fn into_string(input: *const c_char) -> error::Result<String> {
    let raw_buf = unsafe { CStr::from_ptr(input).to_bytes() };
    let res = String::from_utf8(raw_buf.to_vec())?;
    Ok(res)
}

#[no_mangle]
pub extern "C" fn release(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return
        }
        CString::from_raw(s)
    };
}

fn error_to_cstring(e: error::Error) -> CString {
    let e_data = format!("{:?}", e);
    CString::new(e_data).expect("into_cstring: CString::new crash")
}

fn io_error_to_cstring(e: io::Error) -> CString {
    let e_data = format!("{:?}", e);
    CString::new(e_data).expect("io_error_to_cstring: CString::new crash")
}

fn ok_cstring() -> CString {
    CString::new("").expect("ok_cstring: CString::new crash")
}

#[no_mangle]
pub extern "C" fn initialize_target_dir(dir_cstr: *const c_char, need_clean: bool) -> *mut c_char {
    let dir_in = match into_string(dir_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    match util::init_output_dir(Path::new(&dir_in), need_clean) {
        Err(e) => io_error_to_cstring(e).into_raw(),
        Ok(_) => ok_cstring().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn generate_sample_file(expected_size: u32, path_cstr: *const c_char) -> *mut c_char {
    let path_in = match into_string(path_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    match util::gen_sample_file::<PedersenHasher>(expected_size as usize, Path::new(&path_in)) {
        Err(e) => io_error_to_cstring(e).into_raw(),
        Ok(_) => ok_cstring().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn generate_replica_id() -> *mut c_char {
    let replica_id = param::new_replica_id::<PedersenHasher>();
    let replica_id_json = match param::replica_id_into_json::<PedersenHasher>(replica_id) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let res = CString::new(replica_id_json)
        .expect("generate_replica_id: CString::new crash");
    res.into_raw()
}

#[no_mangle]
pub extern "C" fn count_node_num(path_cstr: *const c_char) -> u32 {
    let path_in = match into_string(path_cstr) {
        Err(_) => return 0,
        Ok(v) => v
    }; 

    match util::count_nodes(Path::new(&path_in)) {
        Err(_) => return 0,
        Ok(n) => n as u32
    }
}

#[no_mangle]
pub extern "C" fn generate_setup_params(node_num: u32) -> *mut c_char {
    let sp = param::new_setup_params_with_defaults(node_num as usize);
    let sp_data = match param::setup_params_into_json(&sp) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    
    let res = CString::new(sp_data)
        .expect("generate_setup_params: CString::new crash");
    res.into_raw()
}

#[no_mangle]
pub extern "C" fn generate_store_config(node_num: u32, dir_cstr: *const c_char) -> *mut c_char {
    let dir_in = match into_string(dir_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let scfg = param::new_store_cfg_with_defaults(node_num as usize, Path::new(&dir_in));
    let scfg_data = match param::into_json(&scfg) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };

    let res = CString::new(scfg_data)
        .expect("generate_store_config: CString::new crash");
    res.into_raw()
}

#[no_mangle]
pub extern "C" fn porep_setup(
    src_path_cstr: *const c_char, 
    sp_data_cstr: *const c_char,
    scfg_data_cstr: *const c_char,
    replica_id_cstr: *const c_char,
) -> *mut c_char {
    let src_path = match into_string(src_path_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let src = Path::new(&src_path);

    let _sp_data = match into_string(sp_data_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let sp = match param::setup_params_from_json(&_sp_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let _scfg_data = match into_string(scfg_data_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let scfg = match param::store_cfg_from_json(&_scfg_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };
    
    let _replica_id_data = match into_string(replica_id_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let replica_id = match param::replica_id_from_json::<BinaryMerkleTree<PedersenHasher>>(&_replica_id_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let replica_path = match vanilla::setup_inner::<BinaryMerkleTree<PedersenHasher>>(
        src, 
        &sp,
        &scfg,
        &replica_id
    ) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    // WARNING: 
    // the as_bytes() is an OS-speicified implementation, it does not work on WINDOWS
    let res = CString::new(
            replica_path
                .as_path()
                .as_os_str()
                .as_bytes()
        ).expect("porep_setup: CString::new crash");
    res.into_raw()
}

#[no_mangle]
pub extern "C" fn generate_challenge() -> *mut c_char {
    let chal = param::new_chal_seed();
    let chal_data = match param::into_json(&chal) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };

    let res = CString::new(chal_data)
        .expect("generate_challenge: CString::new crash");
    res.into_raw()
}

// TODO: to discuss whether it is necessary 
// here we just dump the proof data to a path-specified file,
// not sure if returning the 64k json string data back to the go
// code directly through ffi is a good idea
#[no_mangle]
pub extern "C" fn porep_prove(
    replica_path_cstr: *const c_char,
    sp_data_cstr: *const c_char,
    replica_id_cstr: *const c_char,
    chal_cstr: *const c_char,
    proof_path_cstr: *const c_char,
) -> *mut c_char {
    let _replica_path = match into_string(replica_path_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let replica_file = Path::new(&_replica_path);

    let _sp_data = match into_string(sp_data_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let sp = match param::setup_params_from_json(&_sp_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let _replica_id_data = match into_string(replica_id_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let replica_id = match param::replica_id_from_json::<BinaryMerkleTree<PedersenHasher>>(&_replica_id_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let _chal_data = match into_string(chal_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };
    let chal = match param::chal_seed_from_json(&_chal_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let _proof_path = match into_string(proof_path_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let proof_path = Path::new(&_proof_path);

    let proof = match vanilla::prove_inner::<BinaryMerkleTree<PedersenHasher>>(
        replica_file,
        &sp,
        replica_id,
        chal,
        0, // we treat the original replica as 1 partition only data block, therefore 0 is the partition index
        1, // the partition number
    ) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let proof_json = match param::into_json(&proof) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(d) => d
    };

    match util::write_file(proof_path, proof_json.as_bytes()) {
        Err(e) => return io_error_to_cstring(e).into_raw(),
        Ok(_) => ok_cstring().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn porep_verify(
    replica_path_cstr: *const c_char,
    sp_data_cstr: *const c_char,
    replica_id_cstr: *const c_char,
    chal_cstr: *const c_char,
    proof_path_cstr: *const c_char,
) -> *mut c_char {
    let _replica_path = match into_string(replica_path_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let replica_file = Path::new(&_replica_path);

    let _sp_data = match into_string(sp_data_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let sp = match param::setup_params_from_json(&_sp_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let _replica_id_data = match into_string(replica_id_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let replica_id = match param::replica_id_from_json::<BinaryMerkleTree<PedersenHasher>>(&_replica_id_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let _chal_data = match into_string(chal_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };
    let chal = match param::chal_seed_from_json(&_chal_data) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(v) => v
    };

    let _proof_path = match into_string(proof_path_cstr) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };
    let proof = match param::load_proof::<BinaryMerkleTree<PedersenHasher>>(Path::new(&_proof_path)) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(s) => s
    };

    let pass = match vanilla::verify_inner::<BinaryMerkleTree<PedersenHasher>>(
        replica_file,
        &sp,
        replica_id,
        chal,
        0, // the partition index
        &proof,
    ) {
        Err(e) => return error_to_cstring(e).into_raw(),
        Ok(b) => b
    };

    let res = match pass {
        false => CString::new("verification failed").expect("porep_verify: CString::new crash"),
        true => ok_cstring()
    };
    res.into_raw()
}