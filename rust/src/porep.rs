use std::path::{Path, PathBuf};
use std::fs;

use merkletree::store::StoreConfig;
use storage_proofs::cache_key::CacheKey;
use storage_proofs::util::default_rows_to_discard;
use storage_proofs::drgraph::BASE_DEGREE;
use storage_proofs::hasher::{Domain, Hasher, Sha256Hasher};
use storage_proofs::merkle::{BinaryMerkleTree, MerkleTreeTrait};
use storage_proofs::porep::stacked::{self, LayerChallenges, SetupParams, EXP_DEGREE, BINARY_ARITY, StackedDrg, Tau, PersistentAux, TemporaryAux, TemporaryAuxCache};
use storage_proofs::porep::PoRep;
use storage_proofs::proof::ProofScheme;

use super::error::Result;
use super::param::{self, PersistentSetupParam, PersistentTau};
use super::util;

fn dump_setup_inputs<D>(target: &Path, scfg: &StoreConfig, sp: &SetupParams, rid: &D) -> Result<()> 
where
    D: Domain,
{
    let scfg_data = param::into_json(scfg)?;
    util::write_file(&target.with_extension("store_conf").as_path(), scfg_data.as_bytes())?;

    let p_sp = param::PersistentSetupParam::from(sp);
    let p_sp_data = param::into_json(&p_sp)?;
    util::write_file(&target.with_extension("p_sp"), p_sp_data.as_bytes())?;

    let rid_data = rid.into_bytes();
    util::write_file(&target.with_extension("replica_id"), &rid_data)?;

    Ok(())
}

fn restore_setup_params(target: &Path) -> Result<(StoreConfig, SetupParams)> {
    let scfg_data = fs::read_to_string(&target.with_extension("store_conf"))?;
    let scfg = param::from_json::<StoreConfig>(&scfg_data)?;

    let p_sp_data = fs::read_to_string(&target.with_extension("p_sp"))?;
    let p_sp = param::from_json::<PersistentSetupParam>(&p_sp_data)?;
    
    Ok((scfg, SetupParams::from(&p_sp)))
}

fn dump_setup_outputs<D, E, F, T, H>(
    target: &Path,
    tau: &Tau<D, E>, p_aux: &PersistentAux<F>, t_aux: &TemporaryAux<T, H>,
) -> Result<()> 
where
    D: Domain,
    E: Domain,
    F: Domain,
    T: MerkleTreeTrait,
    H: Hasher,
{
    let p_tau = param::PersistentTau::from(tau);
    let p_tau_data = param::into_json(&p_tau)?;
    util::write_file(&target.with_extension("p_tau"), p_tau_data.as_bytes())?;

    let p_aux_data = param::into_json(p_aux)?;
    util::write_file(&target.with_extension("p_aux"), p_aux_data.as_bytes())?;

    let t_aux_data = param::into_json(t_aux)?;
    util::write_file(&target.with_extension("t_aux"), t_aux_data.as_bytes())?;

    Ok(())
}

fn prepare_setup(src: &Path, cache: &Path, id: [u8;32]) -> Result<(StoreConfig, SetupParams)> {
    let nodes = util::count_nodes(src)?;

    Ok((StoreConfig::new(
        cache,
        CacheKey::CommDTree.to_string(),
        default_rows_to_discard(nodes, BINARY_ARITY),
    ), 
    SetupParams{
        nodes,
        degree: BASE_DEGREE,
        expansion_degree: EXP_DEGREE,
        porep_id: id,
        layer_challenges: LayerChallenges::new(param::DEFAULT_LAYER, param::DEFAULT_MAX_COUNT),
    }))
}

pub fn setup_inner<H>(src_path: &Path, cache_path: &Path) -> Result<PathBuf>
where
    H: 'static + Hasher,
{
    let (scfg, sp) = prepare_setup(src_path, cache_path, util::new_seed())?;
    
    let rng = &mut rand::thread_rng();
    let replica_id = H::Domain::random(rng);

    let output_path = util::output_file_name(src_path, cache_path, "replica")?;
    dump_setup_inputs(output_path.as_path(), &scfg, &sp, &replica_id)?;

    let pp = StackedDrg::<BinaryMerkleTree<H>, Sha256Hasher>::setup(&sp)?;
    let data = util::read_file_as_mmap(src_path)?;
    let mut mapped_data = util::write_file_and_mmap(output_path.as_path(), &data)?;
    
    let (tau, (p_aux, t_aux)) =
        StackedDrg::<BinaryMerkleTree<H>, Sha256Hasher>::replicate(
            &pp,
            &replica_id,
            (&mut mapped_data[..]).into(),
            None,
            scfg.clone(),
            output_path.clone(),
        )?;

    dump_setup_outputs(output_path.as_path(), &tau, &p_aux, &t_aux)?;

    Ok(output_path)
}

pub fn prove_inner<H: 'static>(output_path: &Path) -> Result<String>
where
    H: Hasher,
{
    let (scfg, sp) = restore_setup_params(output_path)?;
    let pp = StackedDrg::<BinaryMerkleTree<H>, Sha256Hasher>::setup(&sp)?;

    let replica_id_data = fs::read_to_string(&output_path.with_extension("replica_id").as_path())?;
    let replica_id = param::restore_replica_id::<BinaryMerkleTree<H>>(&replica_id_data)?;

    let p_tau_data = fs::read_to_string(&output_path.with_extension("p_tau").as_path())?;
    let p_tau = serde_json::from_str::<PersistentTau>(&p_tau_data)?;
    let tau = p_tau.as_tau::<BinaryMerkleTree<H>, Sha256Hasher>()?;

    let t_aux_data = fs::read_to_string(&output_path.with_extension("t_aux").as_path())?;
    let t_aux = param::restore_t_aux::<BinaryMerkleTree<H>, Sha256Hasher>(&t_aux_data)?;

    let p_aux_data = fs::read_to_string(&output_path.with_extension("p_aux").as_path())?;
    let p_aux = param::restore_p_aux::<BinaryMerkleTree<H>>(&p_aux_data)?;

    let pb = stacked::PublicInputs::<H::Domain, <Sha256Hasher as Hasher>::Domain> {
        replica_id,
        seed: util::new_seed(),
        tau: Some(tau),
        k: Some(param::DEFAULT_K),
    };

    let t_aux = TemporaryAuxCache::new(&t_aux, output_path.to_path_buf())?;
    let pv = stacked::PrivateInputs {
        p_aux,
        t_aux,
    };

    let proof = StackedDrg::<BinaryMerkleTree<H>, Sha256Hasher>::prove_all_partitions(
        &pp,
        &pb,
        &pv,
        param::DEFAULT_PARTITION,
    )?;

    param::into_json(&proof)
}

// TODO: implement following
// verify_inner

#[cfg(test)]
mod test {
    use storage_proofs::hasher::PedersenHasher;

    use super::*;
    use super::super::util::{self, test::gen_sample_file};

    #[test]
    fn test_setup() {
        let sample_dir = Path::new("./sample");
        
        util::init_output_dir(sample_dir, true)
            .expect("error setting up the test sample dir");

        let input_size: usize = 1024; // 1k, just a simple quick test here
        let input_path = sample_dir.join("sample.dat");

        gen_sample_file::<PedersenHasher>(input_size / 32, input_path.as_path()).unwrap();

        setup_inner::<PedersenHasher>(input_path.as_path(), sample_dir)
            .expect("failed to setup");
    }

    #[test]
    fn test_prove() {
        // TODO: uncouple this case with the test_setup test
        // TODO: move this alone with the big sample generation test to integrate test

        let sample_path = Path::new("./sample/sample.replica");
        let res = prove_inner::<PedersenHasher>(sample_path)
            .expect("failed to prove");

        println!("len(proof) = {}", res.len());
    }
}