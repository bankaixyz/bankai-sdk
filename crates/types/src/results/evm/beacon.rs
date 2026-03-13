use alloy_primitives::FixedBytes;
#[cfg(feature = "inputs")]
use alloy_rpc_types_beacon::header::HeaderResponse;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use tree_hash_derive::TreeHash;

/// Verified Ethereum beacon header.
#[derive(TreeHash, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BeaconHeader {
    /// Beacon slot.
    pub slot: u64,
    /// Beacon proposer index.
    pub proposer_index: u64,
    /// Parent root committed by the beacon header.
    pub parent_root: FixedBytes<32>,
    /// Beacon state root.
    pub state_root: FixedBytes<32>,
    /// Beacon block body root.
    pub body_root: FixedBytes<32>,
}

#[cfg(feature = "inputs")]
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
