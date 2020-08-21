use std::convert::{AsRef, From};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use storage_proofs::hasher::{Hasher, Domain};
use storage_proofs::merkle::{MerkleTreeTrait};
use storage_proofs::porep::stacked::{LayerChallenges, SetupParams, Tau, PersistentAux, TemporaryAux};

use super::error::Result;

// https://github.com/filecoin-project/rust-fil-proofs/blob/storage-proofs-v4.0.1/fil-proofs-tooling/src/bin/benchy/main.rs#L18
// here we use the `benchy` default parameters as constant
// layers = 11
// partitions = 1
// challenges = 1
// k = 0

pub const DEFAULT_LAYER: usize = 11;
pub const DEFAULT_MAX_COUNT: usize = 1;
pub const DEFAULT_PARTITION: usize = 1;
pub const DEFAULT_K: usize = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentSetupParam {
    pub nodes: usize,
    pub degree: usize,
    pub expansion_degree: usize,
    pub porep_id: [u8; 32],
    pub layer_challenges: LayerChallenges,
}

impl<'a> From<&'a SetupParams> for PersistentSetupParam {
    fn from(sp: &'a SetupParams) -> Self {
        PersistentSetupParam {
            nodes: sp.nodes,
            degree: sp.degree,
            expansion_degree: sp.expansion_degree,
            porep_id: sp.porep_id.clone(),
            layer_challenges: sp.layer_challenges.clone(),
        }
    }
}

impl<'a> From<&'a PersistentSetupParam> for SetupParams {
    fn from(p_sp: &'a PersistentSetupParam) -> Self {
        SetupParams {
            nodes: p_sp.nodes,
            degree: p_sp.degree,
            expansion_degree: p_sp.expansion_degree,
            porep_id: p_sp.porep_id.clone(),
            layer_challenges: p_sp.layer_challenges.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentTau {
    pub comm_d: Vec<u8>,
    pub comm_r: Vec<u8>,
}

impl<'a, D: Domain, E: Domain> From<&'a Tau<D, E>> for PersistentTau {
    fn from(tau: &'a Tau<D, E>) -> Self {
        PersistentTau {
            comm_d: tau.comm_d.into_bytes(),
            comm_r: tau.comm_r.into_bytes(),
        }
    }
}

impl PersistentTau {
    pub fn as_tau<Tree, G>(&self) -> Result<Tau<<Tree::Hasher as Hasher>::Domain, <G as Hasher>::Domain>>
    where
        Tree: 'static + MerkleTreeTrait,
        G: 'static + Hasher,
    {
        let comm_d = Domain::try_from_bytes(self.comm_d.as_ref())?;
        let comm_r = Domain::try_from_bytes(self.comm_r.as_ref())?;

        Ok(Tau {
            comm_d,
            comm_r,
        })
    }
}

pub fn into_json<T: Serialize>(param: &T) -> Result<String> {
    let data = serde_json::to_string(param)?;
    Ok(data)
}

pub fn from_json<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T> {
    let inst = serde_json::from_str(s)?;
    Ok(inst)
}

pub fn restore_replica_id<'a, Tree>(s: &'a str) -> Result<<Tree::Hasher as Hasher>::Domain>
where
    Tree: 'static + MerkleTreeTrait,
{
    from_json(s)
}

pub fn restore_p_aux<'a, Tree>(s: &'a str) -> Result<PersistentAux<<Tree::Hasher as Hasher>::Domain>>
where
    Tree: 'static + MerkleTreeTrait,
{
    from_json(s)
}

pub fn restore_t_aux<'a, Tree, G>(s: &'a str) -> Result<TemporaryAux<Tree, G>>
where
    Tree: 'static + MerkleTreeTrait,
    G: 'static + Hasher,
{
    from_json(s)
}

#[cfg(test)]
mod test {
    use rand::prelude::*;
    use serde_json;

    use storage_proofs::drgraph::BASE_DEGREE;
    use storage_proofs::hasher::{PedersenHasher, Sha256Hasher};
    use storage_proofs::merkle::BinaryMerkleTree;
    use storage_proofs::porep::stacked::{EXP_DEGREE, LayerChallenges, SetupParams};

    use super::*;
    use super::super::util;

    fn sample_setup_params() -> SetupParams {
        SetupParams {
            nodes: random(),
            degree: BASE_DEGREE,
            expansion_degree: EXP_DEGREE,
            porep_id: util::new_seed(),
            layer_challenges: LayerChallenges::new(DEFAULT_LAYER, DEFAULT_MAX_COUNT),
        }
    }

    #[test]
    fn test_serde_setup_params() {
        let lhs = sample_setup_params();
        let tmp = PersistentSetupParam::from(&lhs);

        let data = serde_json::to_string(&tmp).unwrap();
        let another_tmp: PersistentSetupParam = serde_json::from_str(&data).unwrap();

        let rhs = SetupParams::from(&another_tmp);

        assert_eq!(lhs.nodes, rhs.nodes);
        assert_eq!(lhs.degree, rhs.degree);
        assert_eq!(lhs.expansion_degree, rhs.expansion_degree);
        assert_eq!(&lhs.porep_id, &rhs.porep_id);
        assert_eq!(lhs.layer_challenges.layers(), rhs.layer_challenges.layers());
        assert_eq!(lhs.layer_challenges.challenges_count_all(), rhs.layer_challenges.challenges_count_all());
    }

    #[test]
    fn test_serde_replica_id() {
        let sample_replica_id_data = r#"[6868552744863790462,18061126747641064871,15229175371992025091,5445982386805143806]"#;

        let replica_id = restore_replica_id::<BinaryMerkleTree<PedersenHasher>>(sample_replica_id_data)
            .expect("error restore replica ID object");
        let another_replica_id_data = into_json(&replica_id)
            .expect("error dump the ref replica ID object");
        
            assert_eq!(sample_replica_id_data, another_replica_id_data);
    }

    #[test]
    fn test_serde_replicate_outputs() {
        let sample_tau_data = r#"{"comm_d":[47,253,158,87,141,16,90,22,36,36,99,164,198,37,99,1,56,137,103,149,86,151,239,199,164,211,19,207,238,145,27,2],"comm_r":[86,191,113,180,80,26,122,13,59,114,233,214,250,72,193,162,99,72,196,77,137,250,225,79,56,106,225,169,218,66,51,15]}"#;
        let sample_p_aux_data = r#"{"comm_c":[17267550153351710043,10886907006776105359,3945427731707610239,4099319404227195],"comm_r_last":[7436427305259939742,6382606002039802184,15244580351193237470,3087863950776047946]}"#;
        let sample_t_aux_data = r#"{"labels":{"labels":[{"path":"./sample","id":"layer-1","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-2","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-3","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-4","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-5","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-6","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-7","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-8","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-9","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-10","size":32,"rows_to_discard":4},{"path":"./sample","id":"layer-11","size":32,"rows_to_discard":4}],"_h":null},"tree_d_config":{"path":"./sample","id":"tree-d","size":63,"rows_to_discard":4},"tree_r_last_config":{"path":"./sample","id":"tree-r-last","size":63,"rows_to_discard":4},"tree_c_config":{"path":"./sample","id":"tree-c","size":63,"rows_to_discard":4},"_g":null}"#;

        let p_tau = serde_json::from_str::<PersistentTau>(sample_tau_data)
            .expect("error restore PersistentTau object");
        let tau = p_tau.as_tau::<BinaryMerkleTree<PedersenHasher>, Sha256Hasher>()
            .expect("as_tau: type convert failed");
        let p_aux = restore_p_aux::<BinaryMerkleTree<PedersenHasher>>(sample_p_aux_data)
            .expect("error restore PersistentAux object");
        let t_aux = restore_t_aux::<BinaryMerkleTree<PedersenHasher>, Sha256Hasher>(sample_t_aux_data)
            .expect("error restore TemporaryAux object");

        let another_p_tau = PersistentTau::from(&tau);
        let another_p_tau_data = into_json(&another_p_tau)
            .expect("error dump the ref PersistentTau object");
        let another_p_aux_data = into_json(&p_aux)
            .expect("error dump the ref PersistentAux object");
        let another_t_aux_data = into_json(&t_aux)
            .expect("error dump the ref TemporaryAux object");

        assert_eq!(sample_tau_data, another_p_tau_data);
        assert_eq!(sample_p_aux_data, another_p_aux_data);
        assert_eq!(sample_t_aux_data, another_t_aux_data);
    }
}