extern crate libc;

use std::convert::TryInto;
use std::ffi::CStr;
use std::fs::metadata;
use std::path::Path;
use std::result::Result;

use merkletree::store::StoreConfig;
use rand;
use storage_proofs::cache_key::CacheKey;
use storage_proofs::util::default_rows_to_discard;
use storage_proofs::drgraph::BASE_DEGREE;
use storage_proofs::hasher::{Domain, Hasher, PedersenHasher, Sha256Hasher};
use storage_proofs::merkle::BinaryMerkleTree;
use storage_proofs::porep::stacked::{LayerChallenges, StackedDrg, SetupParams, EXP_DEGREE, BINARY_ARITY};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;

use super::util;
use super::param;

const OK: u32 = 0;
const ERR_RET: u32 = std::u32::MAX;

fn get_nodes_num<P: AsRef<Path>>(path: P) -> Result<usize, String> {
    match metadata(path) {
        Err(e) => {
            return Err(format!("failed to open file: {}", e));
        },
        Ok(meta) => {
            // TODO: magic number
            match (meta.len() / 32).try_into() {
                Err(e) => {
                    return Err(format!("error convert: {}", e));
                },
                Ok(c) => { return Ok(c); },
            }
        }
    }
}

// https://github.com/filecoin-project/rust-fil-proofs/blob/storage-proofs-v4.0.1/fil-proofs-tooling/src/bin/benchy/main.rs#L18
// here we use the `benchy` default parameters as constant
// layers = 11
// hasher = pedersen
// partitions = 1
// challenges = 1

fn setup_inner<H: 'static>(data_path: &str, cache_path: &str) -> Result<u32, String>
where
    H: Hasher,
{
    let nodes = match get_nodes_num(data_path) {
        Err(e) => {
            return Err(format!("error get file's metadata: {}", e))
        },
        Ok(c) => c,
    };

    let cfg = StoreConfig::new(
        cache_path, 
        CacheKey::CommDTree.to_string(), 
        default_rows_to_discard(nodes, BINARY_ARITY),
    );

    let sp = SetupParams{
        nodes: nodes,
        degree: BASE_DEGREE,
        expansion_degree: EXP_DEGREE,
        porep_id: util::new_seed(),
        layer_challenges: LayerChallenges::new(11, 1),
    };

    match param::dump_setup_param(&sp) {
        Err(e) => {
            return Err(format!("error generating setup param: {}", e))
        },
        Ok(data) => {
            println!("SetupParams = {}", data.as_str());
        },
    }

    let pp = match StackedDrg::<BinaryMerkleTree<H>, Sha256Hasher>::setup(&sp) {
        Err(e) => {
            return Err(format!("error setting up: {}", e))
        },
        Ok(v) => v,
    };

    let rng = &mut rand::thread_rng();
    let replica_id = H::Domain::random(rng);

    let data = match util::read_file_as_mmap(Path::new(data_path)) {
        Err(e) => {
            return Err(format!("error read data: {}", e))
        },
        Ok(d) => d,
    };

    // TODO: replace the fixed replica file name by "$(FILENAME).replica" later
    // TODO: more path validation logic here
    let replica_file = Path::new(cache_path).join("replica.dat");
    let mut mapped_data = match util::write_file_and_mmap(replica_file.as_path(), &data) {
        Err(e) => {
            return Err(format!("error creating replica file: {}", e))
        },
        Ok(d) => d,
    };
    
    let replicate_res =
        StackedDrg::<BinaryMerkleTree<H>, Sha256Hasher>::replicate(
            &pp,
            &replica_id,
            (&mut mapped_data[..]).into(),
            None,
            cfg.clone(),
            replica_file.clone(),
        );
    let (tau, (p_aux, t_aux)) = match replicate_res {
        Err(e) => {
            return Err(format!("error replicate data: {}", e))
        },
        Ok(t) => t,
    };

    match param::dump_tau(&tau) {
        Err(e) => {
            return Err(format!("error dump param tau: {}", e))
        },
        Ok(data) => {
            println!("tau = {}", data);
        },
    }

    match param::dump_p_aux(&p_aux) {
        Err(e) => {
            return Err(format!("error dump param p_aux: {}", e))
        },
        Ok(data) => {
            println!("p_aux = {}", data);
        },
    }

    match param::dump_t_aux(&t_aux) {
        Err(e) => {
            return Err(format!("error dump param t_aux: {}", e))
        },
        Ok(data) => {
            println!("t_aux = {}", data);
        },
    }

    Ok(OK)
}

// FFI
#[no_mangle]
pub extern "C" fn setup(data_path: *const libc::c_char, cache_dir: *const libc::c_char) -> u32 {
    let file_path_buf = unsafe { CStr::from_ptr(data_path).to_bytes() };
    let file_path = match String::from_utf8(file_path_buf.to_vec()) {
        Err(e) => {
            eprintln!("invalid data file path: {}", e);
            return ERR_RET
        },
        Ok(p) => p,
    };

    let cache_path_buf = unsafe { CStr::from_ptr(cache_dir).to_bytes() };
    let cache_path = match String::from_utf8(cache_path_buf.to_vec()) {
        Err(e) => {
            eprintln!("invalid data file path: {}", e);
            return ERR_RET
        },
        Ok(p) => p,
    };

    let res = match setup_inner::<PedersenHasher>(file_path.as_str(), cache_path.as_str()) {
        Err(info) => {
            eprintln!("error setup: {}", info);
            return ERR_RET
        },
        Ok(v) => v,
    };

    res
}

#[cfg(test)]
mod test {
    use std::fs::{self, remove_file};

    use storage_proofs::hasher::PedersenHasher;

    use super::*;
    use super::super::util::test::gen_sample_file;

    #[test]
    fn test_gen_sample_data() {
        let input_size: usize = 1024 * 1024;
        let input_path = Path::new("sample.txt");

        let gen_result = gen_sample_file::<PedersenHasher>(input_size / 32, input_path);
        let gen_size = metadata(input_path).unwrap().len();

        assert_eq!(input_size, gen_size as usize);
        println!("A sample file({}) with {} nodes({} Bytes) generated successfully!", 
            input_path.display(), gen_result.unwrap(), gen_size);

        // comment the following out if you need the generated data
        remove_file(input_path).expect("failed to delete generated sample file.");
    } 

    #[test]
    fn test_setup() {
        let sample_dir = Path::new("./sample");
        fs::create_dir(sample_dir).unwrap();

        let input_size: usize = 1024; // 1k, just a simple quick test here
        let input_path = sample_dir.join("sample.txt");

        gen_sample_file::<PedersenHasher>(input_size / 32, input_path.as_path()).unwrap();

        let res = setup_inner::<PedersenHasher>(input_path.to_str().unwrap(), sample_dir.to_str().unwrap());
        assert!(res.is_ok());
    }
}