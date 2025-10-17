use crate::{fetch::evm::MmrProof, verify::evm::beacon::BeaconHeader};
use serde::{Deserialize, Serialize};

#[cfg_attr(any(feature = "verifier-types", feature = "std"), derive(Debug, Clone))]
#[derive(Serialize, Deserialize)]
pub struct BeaconHeaderProof {
    pub header: BeaconHeader,
    pub mmr_proof: MmrProof,
}
