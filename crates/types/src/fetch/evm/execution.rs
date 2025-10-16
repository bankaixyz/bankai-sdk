extern crate alloc;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::proofs::MmrProofDto;
use alloy_primitives::{Address, Bytes, FixedBytes};

#[cfg(feature = "verifier-types")]
use alloy_rpc_types_eth::{Account, Header as ExecutionHeader};

#[cfg(feature = "verifier-types")]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ExecutionHeaderProof {
    pub header: ExecutionHeader,
    pub mmr_proof: MmrProofDto,
}

#[cfg(feature = "verifier-types")]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct AccountProof {
    pub account: Account,
    pub address: Address,
    pub network_id: u64,
    pub block_number: u64,
    pub state_root: FixedBytes<32>,
    pub mpt_proof: Vec<Bytes>,
}

#[cfg(feature = "verifier-types")]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct TxProof {
    pub network_id: u64,
    pub block_number: u64,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: u64,
    pub proof: Vec<Bytes>,
    pub encoded_tx: Vec<u8>,
}
