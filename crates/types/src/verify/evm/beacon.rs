use alloy_primitives::FixedBytes;
use serde::{Deserialize, Serialize};
use tree_hash_derive::TreeHash;

#[cfg(feature = "verifier-types")]
use alloy_rpc_types_beacon::header::HeaderResponse;

#[derive(TreeHash, Clone, Debug, Serialize, Deserialize)]
pub struct BeaconHeader {
    pub slot: u64,
    pub proposer_index: u64,
    pub parent_root: FixedBytes<32>,
    pub state_root: FixedBytes<32>,
    pub body_root: FixedBytes<32>,
}

#[cfg(feature = "verifier-types")]
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
