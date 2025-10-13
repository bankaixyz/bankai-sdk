use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

use crate::fetch::evm::{EvmProofs, EvmProofsRequest};

pub mod evm;

// #[derive(Debug)]
pub struct ProofWrapper {
    pub block_proof: CairoProof<Blake2sMerkleHasher>,
    pub evm_proofs: Option<EvmProofs>,
}

#[derive(Debug)]
pub struct ProofRequest {
    pub evm_proofs: Option<EvmProofsRequest>,
}