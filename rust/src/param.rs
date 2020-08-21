use std::convert::{AsRef, From};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use storage_proofs::hasher::{Hasher, Domain};
use storage_proofs::merkle::{MerkleTreeTrait};
use storage_proofs::porep::stacked::{LayerChallenges, SetupParams, Tau};

use super::error;

// https://github.com/filecoin-project/rust-fil-proofs/blob/storage-proofs-v4.0.1/fil-proofs-tooling/src/bin/benchy/main.rs#L18
// here we use the `benchy` default parameters as constant
// layers = 11
// partitions = 1
// challenges = 1

pub const DEFAULT_LAYER: usize = 11;
pub const DEFAULT_MAX_COUNT: usize = 1;
pub const DEFAULT_PARTITION: usize = 1;

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
    pub fn as_tau<Tree, G>(&self) -> error::Result<Tau<<Tree::Hasher as Hasher>::Domain, <G as Hasher>::Domain>>
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

pub fn into_json<T: Serialize>(param: &T) -> error::Result<String> {
    let data = serde_json::to_string(param)?;
    Ok(data)
}

pub fn from_json<'a, T: Deserialize<'a>>(s: &'a str) -> error::Result<T> {
    let inst = serde_json::from_str(s)?;
    Ok(inst)
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
    fn test_serde_tau() {
        // TODO: uncouple this case with the test_setup test
        let sample_tau_path = Path::new("./sample/sample.p_tau");
        let sample_tau_data = fs::read_to_string(sample_tau_path)
            .expect("error loading the ./sample/sample.p_tau file's data");

        let p_tau = serde_json::from_str::<PersistentTau>(&sample_tau_data)
            .expect("error restore PersistentTau object");
        let tau = p_tau.as_tau::<BinaryMerkleTree<PedersenHasher>, Sha256Hasher>()
            .expect("as_tau: type convert failed");

        let another_p_tau = PersistentTau::from(&tau);
        let another_p_tau_data = serde_json::to_string(&another_p_tau)
            .expect("error dump the ref PersistentTau object");

        assert_eq!(sample_tau_data, another_p_tau_data);
    }
}