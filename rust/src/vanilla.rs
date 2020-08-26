use std::convert::Into;
use std::path::{Path, PathBuf};

use merkletree::store::StoreConfig;
use storage_proofs::hasher::{Domain, Hasher, Sha256Hasher};
use storage_proofs::merkle::{BinaryMerkleTree, MerkleTreeTrait};
use storage_proofs::porep::stacked::{self, SetupParams, StackedDrg, PersistentAux, TemporaryAux, TemporaryAuxCache, Proof};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;

use super::{error::Result, param, util};

pub fn setup_inner<Tree: 'static + MerkleTreeTrait>(
    src_file: &Path, out_dir: &Path, 
    sp: &SetupParams, scfg: &StoreConfig,
    replica_id: &<Tree::Hasher as Hasher>::Domain,
) -> Result<PathBuf> {
    let replica_path = util::target_file_name(src_file, out_dir, param::EXT_REPLICA)?;

    let pp = StackedDrg::<Tree, Sha256Hasher>::setup(sp)?;
    let data = util::read_file_as_mmap(src_file)?;
    let mut mapped_data = util::write_file_and_mmap(replica_path.as_path(), &data)?;
    
    let (tau, (p_aux, t_aux)) = StackedDrg::<Tree, Sha256Hasher>::replicate(
        &pp,
        replica_id,
        (&mut mapped_data[..]).into(),
        None,
        scfg.clone(),
        replica_path.clone(),
    )?;

    // save key parameters
    param::save_tau(&replica_path, &tau)?;
    param::save_param(&replica_path, &p_aux, param::EXT_PERSIST_AUX)?;
    param::save_param(&replica_path, &t_aux, param::EXT_TEMP_AUX)?;

    Ok(replica_path)
}

pub fn prove_inner<Tree: 'static + MerkleTreeTrait>(
    output_path: &Path, 
    sp: &SetupParams, 
    replica_id: <Tree::Hasher as Hasher>::Domain,
    chal_seed: [u8; 32],
    k: usize, // partition index
    partition: usize,
) -> Result<Vec<Vec<Proof<Tree, Sha256Hasher>>>> {
    let pp = StackedDrg::<Tree, Sha256Hasher>::setup(sp)?;

    let tau = param::load_tau::<Tree, Sha256Hasher>(output_path)?;
    let pb = stacked::PublicInputs::<<Tree::Hasher as Hasher>::Domain, <Sha256Hasher as Hasher>::Domain> {
        replica_id,
        seed: chal_seed,
        tau: Some(tau),
        k: Some(k),
    };

    let t_aux = param::load_t_aux::<Tree, Sha256Hasher>(output_path)?;
    let p_aux = param::load_p_aux::<Tree>(output_path)?;
    let t_aux = TemporaryAuxCache::new(&t_aux, output_path.to_path_buf())?;
    let pv = stacked::PrivateInputs {
        p_aux,
        t_aux,
    };

    let proof = StackedDrg::<Tree, Sha256Hasher>::prove_all_partitions(
        &pp,
        &pb,
        &pv,
        partition,
    )?;

    Ok(proof)
}

pub fn verify_inner<Tree: 'static + MerkleTreeTrait>(
    output_path: &Path, 
    sp: &SetupParams,
    replica_id: <Tree::Hasher as Hasher>::Domain,
    chal_seed: [u8; 32],
    k: usize,
    proof: &[Vec<Proof<Tree, Sha256Hasher>>],
) -> Result<bool> {
    let pp = StackedDrg::<Tree, Sha256Hasher>::setup(&sp)?;

    let tau = param::load_tau::<Tree, Sha256Hasher>(output_path)?;
    let pb = stacked::PublicInputs::<<Tree::Hasher as Hasher>::Domain, <Sha256Hasher as Hasher>::Domain> {
        replica_id,
        seed: chal_seed,
        tau: Some(tau),
        k: Some(k),
    };

    let res = StackedDrg::<Tree, Sha256Hasher>::verify_all_partitions(
        &pp,
        &pb,
        &proof,
    )?;

    Ok(res)
}

// TODO: evaluate whether it is necessary to wrap StackedDrg::extract_all()

#[cfg(test)]
mod test {
    // use storage_proofs::hasher::PedersenHasher;

    // use super::*;
    // use super::super::util::{self, test::gen_sample_file};

    // #[test]
    // fn test_setup() {
    //     // TODO: move to integrate test
    //     let sample_dir = Path::new("./sample");
        
    //     util::init_output_dir(sample_dir, true)
    //         .expect("error setting up the test sample dir");

    //     let input_size: usize = 1024; // 1k, just a simple quick test here
    //     let input_path = sample_dir.join("sample.dat");

    //     gen_sample_file::<PedersenHasher>(input_size / 32, input_path.as_path()).unwrap();

    //     setup_inner::<PedersenHasher>(input_path.as_path(), sample_dir)
    //         .expect("failed to setup");
    // }

    // #[test]
    // fn test_prove_and_verify() {
    //     // TODO: move to integrate test
    //     let sample_path = Path::new("./sample/sample.replica");
    //     let seed: [u8; 32] = util::new_seed();

    //     prove_inner::<PedersenHasher>(sample_path, seed.clone())
    //         .expect("failed to prove");
        
    //     let res = verify_inner::<PedersenHasher>(sample_path, seed)
    //         .expect("failed to verify");
    //     assert!(res);
    // }
}