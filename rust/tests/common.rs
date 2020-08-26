extern crate wrapper;

use std::io;
use std::path::Path;

use rand;
use storage_proofs::hasher::{Hasher, Domain};

use wrapper::util;

// helper: create a sample data file
pub fn gen_sample_file<H: Hasher>(want_bytes_num: usize, path: &Path) -> io::Result<()> { 
    let nodes = want_bytes_num / 32;

    let rng = &mut rand::thread_rng();
    let data: Vec<u8> = (0..nodes)
        .flat_map(|_| {
            let v: H::Domain = H::Domain::random(rng);
            v.into_bytes()
        })
        .collect();

    util::write_file(path, &data)
}