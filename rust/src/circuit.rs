// TODO: implement the circuit api

// unit tests
#[cfg(test)]
mod test {
    use std::path::Path;
    use std::time::Instant;

    use bellperson::Circuit;
    use paired::bls12_381::Bls12;
    use rand::rngs;

    use storage_proofs::compound_proof::{self, CompoundProof};
    use storage_proofs::gadgets::BenchCS;
    use storage_proofs::hasher::{Hasher, Sha256Hasher, PedersenHasher};
    use storage_proofs::merkle::{BinaryMerkleTree, MerkleTreeTrait};
    use storage_proofs::porep::stacked::{self, ChallengeRequirements, StackedCompound, StackedDrg, TemporaryAuxCache};
    use storage_proofs::proof::ProofScheme;

    use super::super::{util, param, vanilla};

    const TEST_DIR: &str = "./sample";
    const TEST_SRC: &str = "./sample/sample.dat";

    #[test]
    fn test_circuit_flow_1m() {
        let dir = Path::new(TEST_DIR);
        // init the target directory
        util::init_output_dir(dir, true).unwrap();

        // create a sample source file with random content
        let src = Path::new(TEST_SRC);
        util::gen_sample_file::<PedersenHasher>(1024 * 1024, src).unwrap();

        // replica ID
        let replica_id = param::new_replica_id::<PedersenHasher>();

        sample_circuit_workflow::<BinaryMerkleTree<PedersenHasher>>(
            dir, src, 
            replica_id.clone(),
            param::new_chal_seed(),
        );
    }

    fn sample_circuit_workflow<Tree: 'static + MerkleTreeTrait>(
        dir: &Path,
        src: &Path,
        replica_id: <Tree::Hasher as Hasher>::Domain,
        chal_seed: [u8; 32],
    ) {
        // prepare necessary parameters
        let (scfg, sp) = param::default_setup(src, dir, param::new_porep_id()).unwrap();

        let replica_path = vanilla::setup_inner::<Tree>(
                src,
                &sp, 
                &scfg, 
                &replica_id,
            ).unwrap();

        let pp = StackedDrg::<Tree, Sha256Hasher>::setup(&sp).unwrap();
        let compound_pp = compound_proof::PublicParams {
            vanilla_params: pp.clone(),
            partitions: Some(1),
            priority: false,
        };

        // circuit synthesize, not sure if this part is NECESSARY
        // it seems that just from benchy example, 
        let mut cs = BenchCS::<Bls12>::new();
        <StackedCompound<_, _> as CompoundProof<StackedDrg<Tree, Sha256Hasher>, _>>::blank_circuit(
            &pp,
        )
        .synthesize(&mut cs).unwrap();

        // circuit prove
        // TODO: replace the rng with a seedable rng instance
        // TODO: find a better way to parameterize the input randomizer
        let gp = <StackedCompound<_, _> as CompoundProof<
                StackedDrg<Tree, Sha256Hasher>, 
                _,
            >>::groth_params::<rngs::OsRng>(
                None, 
                &compound_pp.vanilla_params,
            ).unwrap();
        
        let tau = param::load_tau::<Tree, Sha256Hasher>(&replica_path).unwrap();
        let pb = stacked::PublicInputs::<<Tree::Hasher as Hasher>::Domain, <Sha256Hasher as Hasher>::Domain> {
            replica_id,
            seed: chal_seed,
            tau: Some(tau),
            k: Some(0),
            porep_id: sp.porep_id.clone(),
        };

        let t_aux = param::load_t_aux::<Tree, Sha256Hasher>(&replica_path).unwrap();
        let p_aux = param::load_p_aux::<Tree>(&replica_path).unwrap();
        let t_aux = TemporaryAuxCache::new(&t_aux, replica_path.to_path_buf()).unwrap();
        let pv = stacked::PrivateInputs {
            p_aux,
            t_aux,
        };
        
        // prove call
        let start = Instant::now();
        let proof = StackedCompound::prove(&compound_pp, &pb, &pv, &gp).unwrap();
        println!("'prove' costs {:?} ...", start.elapsed());

        // verify call
        let start = Instant::now();
        let result = StackedCompound::verify(
            &compound_pp, 
            &pb, 
            &proof, 
            &ChallengeRequirements {
                minimum_challenges: 1,
            },
        ).unwrap();
        println!("'verify' costs {:?} ...", start.elapsed());

        // result assertion
        assert!(result);
    }
}