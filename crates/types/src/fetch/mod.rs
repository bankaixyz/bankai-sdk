//! Types for fetching and wrapping proofs
//!
//! This module contains types used when fetching proofs from the Bankai API
//! and preparing them for verification. The main type is [`ProofWrapper`],
//! which bundles together all proofs needed for batch verification.

use cairo_air::CairoProof;
use serde::{Deserialize, Serialize};
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

use crate::{
    fetch::evm::{EvmProofs, EvmProofsRequest},
    proofs::HashingFunctionDto,
};

pub mod evm;

/// Complete proof bundle ready for verification
///
/// Contains everything needed for stateless batch verification:
/// - STWO block proof (establishes trust in MMR roots)
/// - Optional EVM proofs (headers, accounts, transactions with their MMR proofs)
///
/// This is the output from the SDK's batch builder and input to the verifier.
#[derive(Serialize, Deserialize)]
pub struct ProofWrapper {
    /// Hash function used for MMR construction
    pub hashing_function: HashingFunctionDto,
    /// STWO zero-knowledge proof for the Bankai block
    pub block_proof: CairoProof<Blake2sMerkleHasher>,
    /// EVM-specific proofs (headers, accounts, transactions)
    pub evm_proofs: Option<EvmProofs>,
}

// #[cfg(feature = "std")]
// impl core::fmt::Debug for ProofWrapper {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         f.debug_struct("ProofWrapper")
//             .field("hashing_function", &self.hashing_function)
//             .field("block_proof", &"<CairoProof>")
//             .field("evm_proofs", &self.evm_proofs)
//             .finish()
//     }
// }

/// Request for a batch of proofs
///
/// Specifies what proofs to fetch from the Bankai API.
#[derive(Debug)]
pub struct ProofRequest {
    /// Optional EVM-specific proof requests
    pub evm_proofs: Option<EvmProofsRequest>,
}
