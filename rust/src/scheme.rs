extern crate libc;

use std::ffi::CStr;
use std::fs::{File, OpenOptions, metadata};
use std::io::{Result, Write};
use std::path::Path;

use memmap::{MmapMut, MmapOptions};
use rand::{rngs::OsRng, Rng};
use storage_proofs::drgraph::BASE_DEGREE;
use storage_proofs::hasher::{PedersenHasher, Sha256Hasher};
use storage_proofs::merkle::BinaryMerkleTree;
use storage_proofs::porep::stacked::{LayerChallenges, StackedDrg, SetupParams, EXP_DEGREE};
use storage_proofs::proof::ProofScheme;

fn new_rand_seed() -> [u8; 32] {
    OsRng.gen()
}

fn load_file_backed_mmap(path: &Path) -> Result<MmapMut> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    unsafe {
        MmapOptions::new()
            .map_mut(&file)
    }
}

fn save_file_backed_mmap(data: &[u8], path: &Path) -> Result<MmapMut> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    file.write_all(data)?;

    unsafe {
        MmapOptions::new()
            .map_mut(&file)
    }
}

// https://github.com/filecoin-project/rust-fil-proofs/blob/storage-proofs-v4.0.1/fil-proofs-tooling/src/bin/benchy/main.rs#L18
// here we use the `benchy` default parameters as constant
// layers = 11
// hasher = pedersen
// partitions = 1
// challenges = 1

// #[no_mangle]
// pub extern "C" fn setup(path: *const libc::c_char) -> *mut libc::c_char {
//     let file_path_buf = unsafe { CStr::from_ptr(path).to_bytes() };
//     let file_path = String::from_utf8(file_path_buf.to_vec()).unwrap();

//     let file_meta = metadata(file_path.as_str()).unwrap();
//     let challenges = LayerChallenges::new(11, 1); // TODO: deal with magic number later

//     let sp = SetupParams {
//         nodes: file_meta.len() as usize,
//         degree: BASE_DEGREE,
//         expansion_degree: EXP_DEGREE,
//         porep_id: new_seed(), // TODO: not sure about this part
//         layer_challenges: challenges.clone(),
//     };

//     let pp = StackedDrg::<BinaryMerkleTree<PedersenHasher>, Sha256Hasher>::setup(&sp).unwrap();
// }

#[cfg(test)]
mod test {
    use super::*;

    use std::fs::remove_file;
    use std::ops::Deref;

    use rand::Rng;
    use storage_proofs::hasher::{Domain, Hasher, PedersenHasher};

    // create the test sample file
    fn gen_sample_file<H: 'static>(nodes: usize, path: &Path) -> Result<usize> 
    where
        H: Hasher,
    { 
        let rng = &mut rand::thread_rng();
        let data: Vec<u8> = (0..nodes)
            .flat_map(|_| {
                let v: H::Domain = H::Domain::random(rng);
                v.into_bytes()
            })
            .collect();
        
        let _ = save_file_backed_mmap(&data, path)
            .expect("error saving temporary sample data");

        Ok(data.len())
    }

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
    fn test_file_backed_mmap() {
        let content = format!("the new generated random is {}.", rand::thread_rng().gen_range(1, 114514));
        let gen_path = Path::new("sample.txt");

        {
            let gen_map = save_file_backed_mmap(content.as_bytes(), gen_path).expect("failed to write sample.txt");
            assert_eq!(content.as_bytes(), gen_map.deref());
        }

        let load_map = load_file_backed_mmap(gen_path).expect("failed to read sample.txt");
        assert_eq!(content.as_bytes(), load_map.deref());

        remove_file(gen_path).expect("failed to delete the sample.txt");
    }
}