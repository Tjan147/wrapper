extern crate wrapper;

mod common;

use std::path::Path;
use std::time::Instant;

use storage_proofs::merkle::{BinaryMerkleTree, MerkleTreeTrait};
use storage_proofs::hasher::{Hasher, PedersenHasher};
use storage_proofs::porep::stacked::SetupParams;

use wrapper::{param, util};

use common::gen_sample_file;

const EXAMPLE_DIR: &str = "./sample";
const EXAMPLE_SRC: &str = "./sample/sample.dat";

// WARNING:
// experiments running shows that StackedDrg::replicate() require the input data's
// is of exact 2^N bytes size. For example, a 10 * 1024 input may cause a porep runtime
// panic while size 16 * 1024/32 * 1024/128 * 1024 works well.
#[test]
fn test_workflow_1k_10c() {
    workflow_inner(1024, 10);
}

#[test]
fn test_workflow_16k_10c() {
    workflow_inner(16 * 1024, 10);
}

#[test]
fn test_workflow_128k_10c() {
    workflow_inner(128 * 1024, 10);
}

#[test]
fn test_workflow_1m_10c() {
    workflow_inner(1024 * 1024, 10);
}

fn workflow_inner(expected_size: usize, chal_num: u8) {
    let dir = Path::new(EXAMPLE_DIR);

    // init the target directory
    util::init_output_dir(dir, true).unwrap();
    
    // create a sample source file with random content
    let src = Path::new(EXAMPLE_SRC);
    gen_sample_file::<PedersenHasher>(expected_size, src).unwrap();

    // prepare necessary parameters
    let replica_id = param::new_replica_id::<PedersenHasher>();
    let (scfg, sp) = param::default_setup(src, dir, param::new_porep_id()).unwrap();

    // porep setup
    let start = Instant::now();
    let replica_path = 
        wrapper::setup_inner::<BinaryMerkleTree<PedersenHasher>>(
            src, dir, 
            &sp, &scfg, 
            &replica_id,
        ).unwrap();
    println!("{}B porep setup costs {:?} ...", expected_size, start.elapsed());

    // porep challenge session
    let start = Instant::now();
    chal_session::<BinaryMerkleTree<PedersenHasher>>(chal_num, &replica_path, &sp, replica_id);
    println!("{} rounds porep challenge sessions costs {:?} ...\n", chal_num, start.elapsed());
}

fn chal_session<Tree: 'static + MerkleTreeTrait>(
    times: u8,
    replica_path: &Path,
    sp: &SetupParams,
    replica_id: <Tree::Hasher as Hasher>::Domain,
) {
    for i in 1..times {
        // create challenge
        let chal = param::new_chal_seed();
    
        // prove
        let start = Instant::now();
        let proof = 
            wrapper::prove_inner::<Tree>(
                replica_path, 
                sp, 
                replica_id, 
                chal.clone(), 
                0, // partition index
                1, // partition count
            ).unwrap();
        println!("[{}] porep prove costs {:?} ;", i, start.elapsed());

        // verify
        let start = Instant::now();

        let is_pass = 
            wrapper::verify_inner::<Tree>(
                replica_path,
                sp,
                replica_id,
                chal,
                0, // partition index
                &proof,
            ).unwrap();
        
        assert!(is_pass);
        println!("[{}] porep verify costs {:?} ;", i, start.elapsed());
    }
}