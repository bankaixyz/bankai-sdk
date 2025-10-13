use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

use crate::{
    fetch::evm::{EvmProofs, EvmProofsRequest},
    proofs::HashingFunctionDto,
};

pub mod evm;

// Manual Debug implementation since CairoProof doesn't implement Debug
pub struct ProofWrapper {
    pub hashing_function: HashingFunctionDto,
    pub block_proof: CairoProof<Blake2sMerkleHasher>,
    pub evm_proofs: Option<EvmProofs>,
}

#[cfg(feature = "std")]
impl core::fmt::Debug for ProofWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProofWrapper")
            .field("hashing_function", &self.hashing_function)
            .field("block_proof", &"<CairoProof>")
            .field("evm_proofs", &self.evm_proofs)
            .finish()
    }
}

#[derive(Debug)]
pub struct ProofRequest {
    pub evm_proofs: Option<EvmProofsRequest>,
}
