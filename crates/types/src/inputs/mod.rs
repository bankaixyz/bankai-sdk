//! Typed verifier inputs assembled by the SDK.

use cairo_air::CairoProof;
use serde::{Deserialize, Serialize};
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

use crate::block::BankaiBlock;
use crate::common::HashingFunction;
use crate::inputs::evm::{EvmProofs, op_stack::OpStackProofs};

pub mod evm;

#[derive(Serialize, Deserialize)]
pub struct ProofBundle {
    pub hashing_function: HashingFunction,
    pub block_proof: CairoProof<Blake2sMerkleHasher>,
    pub block: BankaiBlock,
    pub evm_proofs: Option<EvmProofs>,
    pub op_stack_proofs: Option<OpStackProofs>,
}
