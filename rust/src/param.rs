use std::convert::{AsRef, From};

use serde::{Deserialize, Serialize};
use storage_proofs::hasher::Domain;
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

// TODO: replace following method with TryFrom implementation later
impl PersistentTau {
    pub fn into_tau<D: Domain, E: Domain>(&self) -> error::Result<Tau<D, E>> {
        let comm_d = Domain::try_from_bytes(self.comm_d.as_ref())?;
        let comm_r = Domain::try_from_bytes(self.comm_r.as_ref())?;

        Ok(Tau {
            comm_d,
            comm_r,
        })
    }
}

pub fn dump_as_json<T: Serialize>(param: &T) -> error::Result<String> {
    let data = serde_json::to_string(param)?;
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
}