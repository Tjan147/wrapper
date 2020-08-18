use serde::{Deserialize, Serialize};

use storage_proofs::merkle::MerkleTreeTrait;
use storage_proofs::hasher::{Domain, Hasher};
use storage_proofs::porep::stacked::{LayerChallenges, SetupParams, Tau, PersistentAux, TemporaryAux};

const DEFAULT_LAYER: usize = 11;
const DEFAULT_MAX_COUNT: usize = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentSetupParam {
    pub nodes: usize,
    pub degree: usize,
    pub expansion_degree: usize,
    pub porep_id: [u8; 32],
    pub layer_challenges: LayerChallenges,
}

impl PersistentSetupParam {
    pub fn from_setup_params(sp: &SetupParams) -> Self {
        PersistentSetupParam {
            nodes: sp.nodes,
            degree: sp.degree,
            expansion_degree: sp.expansion_degree,
            porep_id: sp.porep_id.clone(),
            layer_challenges: sp.layer_challenges.clone(),
        }
    }

    pub fn into_setup_params(&self) -> SetupParams {
        SetupParams {
            nodes: self.nodes,
            degree: self.degree,
            expansion_degree: self.expansion_degree,
            porep_id: self.porep_id.clone(),
            layer_challenges: self.layer_challenges.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentTau {
    pub comm_d: String,
    pub comm_r: String,
}

pub fn dump_setup_param(sp: &SetupParams) -> serde_json::Result<String> {
    let p_sp = PersistentSetupParam::from_setup_params(sp);
    let data = serde_json::to_string(&p_sp)?;
    Ok(data)
}

pub fn dump_tau<D: Domain, E: Domain>(tau: &Tau<D, E>) -> serde_json::Result<String> {
    let comm_d = serde_json::to_string(&tau.comm_d)?;
    let comm_r = serde_json::to_string(&tau.comm_r)?;

    let data = serde_json::to_string(&PersistentTau{
        comm_d,
        comm_r,
    })?;
    Ok(data)
}

pub fn dump_p_aux<D: Domain>(p_aux: &PersistentAux<D>) -> serde_json::Result<String> {
    let data = serde_json::to_string(p_aux)?;
    Ok(data)
}

pub fn dump_t_aux<T: MerkleTreeTrait, H: Hasher>(t_aux: &TemporaryAux<T, H>) -> serde_json::Result<String> {
    let data = serde_json::to_string(t_aux)?;
    Ok(data)
}

#[cfg(test)]
mod test {
    use rand::prelude::*;
    use serde_json;

    use storage_proofs::drgraph::BASE_DEGREE;
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
        let tmp = PersistentSetupParam::from_setup_params(&lhs);

        let data = serde_json::to_string(&tmp).unwrap();
        let another_tmp: PersistentSetupParam = serde_json::from_str(&data).unwrap();

        let rhs = another_tmp.into_setup_params();

        assert_eq!(lhs.nodes, rhs.nodes);
        assert_eq!(lhs.degree, rhs.degree);
        assert_eq!(lhs.expansion_degree, rhs.expansion_degree);
        assert_eq!(&lhs.porep_id, &rhs.porep_id);
        assert_eq!(lhs.layer_challenges.layers(), rhs.layer_challenges.layers());
        assert_eq!(lhs.layer_challenges.challenges_count_all(), rhs.layer_challenges.challenges_count_all());
    }
}