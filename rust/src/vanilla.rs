use std::convert::Into;
use std::path::{Path, PathBuf};

use merkletree::store::StoreConfig;
use storage_proofs::hasher::{Hasher, Sha256Hasher};
use storage_proofs::merkle::MerkleTreeTrait;
use storage_proofs::porep::stacked::{self, SetupParams, StackedDrg, TemporaryAuxCache, Proof};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;

use super::{error::Result, param, util};

pub fn setup_inner<Tree: 'static + MerkleTreeTrait>(
    src_file: &Path, 
    sp: &SetupParams, 
    scfg: &StoreConfig,
    replica_id: &<Tree::Hasher as Hasher>::Domain,
) -> Result<PathBuf> {
    let replica_path = util::target_file_name(src_file, &scfg.path, param::EXT_REPLICA)?;

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
    use storage_proofs::merkle::BinaryMerkleTree;
    use storage_proofs::hasher::PedersenHasher;

    use super::*;
    use super::super::param;
    use super::super::util;

    #[test]
    fn test_setup() {
        // TODO: move to integrate test
        let sample_dir = Path::new("./sample");
        
        util::init_output_dir(sample_dir, true)
            .expect("error setting up the test sample dir");

        // WARNING:
        // experiments running shows that StackedDrg::replicate() require the input data's
        // is of exact 2^N bytes size. For example, a 10 * 1024 input may cause a porep runtime
        // panic while size 16 * 1024/32 * 1024/128 * 1024 works well.
        let input_size: usize = 32 * 1024;
        let input_path = sample_dir.join("sample.dat");

        util::gen_sample_file::<PedersenHasher>(input_size / 32, input_path.as_path()).unwrap();

        let replica_id = param::new_replica_id::<PedersenHasher>();
        let (scfg, sp) = param::default_setup(&input_path, sample_dir, param::new_porep_id()).unwrap();

        setup_inner::<BinaryMerkleTree<PedersenHasher>>(
            &input_path,
            &sp, 
            &scfg,
            &replica_id,
        ).expect("failed to setup");
    }
}