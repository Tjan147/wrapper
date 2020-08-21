use std::path::{Path, PathBuf};
use std::fs;

use merkletree::store::StoreConfig;
use storage_proofs::cache_key::CacheKey;
use storage_proofs::util::default_rows_to_discard;
use storage_proofs::drgraph::BASE_DEGREE;
use storage_proofs::hasher::{Domain, Hasher, Sha256Hasher};
use storage_proofs::merkle::{BinaryMerkleTree, MerkleTreeTrait};
use storage_proofs::porep::stacked::{LayerChallenges, SetupParams, EXP_DEGREE, BINARY_ARITY, StackedDrg, Tau, PersistentAux, TemporaryAux};
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

// TODO: implement following
// pub fn prove_inner<H: 'static>(output_path: &Path) -> Result<String>
// where
//     H: Hasher,
// {
// }

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

        let res = setup_inner::<PedersenHasher>(input_path.as_path(), sample_dir);
        assert!(res.is_ok());
    }
}