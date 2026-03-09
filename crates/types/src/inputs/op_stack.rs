extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{FixedBytes, hex::FromHex};
use serde::{Deserialize, Serialize};

use crate::api::op_stack::OpMerkleProofDto;
use crate::block::OpChainClient;
use crate::inputs::evm::MmrProof;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct OpStackProofs {
    pub header_proofs: Vec<OpStackHeaderProof>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct OpStackHeaderProof {
    pub snapshot: OpChainClient,
    pub merkle_proof: OpStackMerkleProof,
    pub mmr_proof: MmrProof,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct OpStackMerkleProof {
    pub chain_id: u64,
    pub merkle_leaf_index: u64,
    pub leaf_hash: FixedBytes<32>,
    pub root: FixedBytes<32>,
    pub path: Vec<FixedBytes<32>>,
}

impl From<OpMerkleProofDto> for OpStackMerkleProof {
    fn from(value: OpMerkleProofDto) -> Self {
        Self {
            chain_id: value.chain_id,
            merkle_leaf_index: value.merkle_leaf_index,
            leaf_hash: FixedBytes::from_hex(value.leaf_hash).unwrap(),
            root: FixedBytes::from_hex(value.root).unwrap(),
            path: value
                .path
                .iter()
                .map(|hash| FixedBytes::from_hex(hash).unwrap())
                .collect(),
        }
    }
}
