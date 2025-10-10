use alloy_primitives::FixedBytes;
use alloy_rpc_types_beacon::header::HeaderResponse;
use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use tree_hash_derive::TreeHash;

use crate::api::MmrProofDto;

pub struct BeaconHeaderProof {
    pub header: BeaconHeader,
    pub block_proof: CairoProof<Blake2sMerkleHasher>,
    pub mmr_proof: MmrProofDto,
}

/// Represents a beacon chain block header
#[derive(TreeHash, Clone, Debug)]
pub struct BeaconHeader {
    /// Slot number of the block
    pub slot: u64,
    /// Index of the block proposer
    pub proposer_index: u64,
    /// Root hash of the parent block
    pub parent_root: FixedBytes<32>,
    /// Root hash of the state
    pub state_root: FixedBytes<32>,
    /// Root hash of the block body
    pub body_root: FixedBytes<32>,
}

impl From<HeaderResponse> for BeaconHeader {
    fn from(header: HeaderResponse) -> Self {
        Self {
            slot: header.data.header.message.slot,
            proposer_index: header.data.header.message.proposer_index,
            parent_root: header.data.header.message.parent_root,
            state_root: header.data.header.message.state_root,
            body_root: header.data.header.message.body_root,
        }
    }
}