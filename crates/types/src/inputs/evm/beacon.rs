use alloy_rpc_types_beacon::header::HeaderResponse;
use serde::{Deserialize, Serialize};

use crate::inputs::evm::MmrProof;

#[cfg_attr(feature = "std", derive(Debug, Clone))]
#[derive(Serialize, Deserialize)]
pub struct BeaconHeaderProof {
    pub header: HeaderResponse,
    pub mmr_proof: MmrProof,
}
