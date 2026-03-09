extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{FixedBytes, hex::FromHex};
use serde::{Deserialize, Serialize};

use crate::api::proofs::MmrProofDto;
use crate::common::HashingFunction;
use crate::inputs::evm::{
    beacon::BeaconHeaderProof,
    execution::{AccountProof, ExecutionHeaderProof, ReceiptProof, StorageSlotProof, TxProof},
};

pub mod beacon;
pub mod execution;
pub mod op_stack;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EvmProofs {
    pub execution_header_proof: Option<Vec<ExecutionHeaderProof>>,
    pub beacon_header_proof: Option<Vec<BeaconHeaderProof>>,
    pub account_proof: Option<Vec<AccountProof>>,
    pub storage_slot_proof: Option<Vec<StorageSlotProof>>,
    pub tx_proof: Option<Vec<TxProof>>,
    pub receipt_proof: Option<Vec<ReceiptProof>>,
}

impl From<MmrProofDto> for MmrProof {
    fn from(mmr_proof: MmrProofDto) -> Self {
        MmrProof {
            network_id: mmr_proof.network_id,
            block_number: mmr_proof.block_number,
            hashing_function: mmr_proof.hashing_function,
            header_hash: FixedBytes::from_hex(mmr_proof.header_hash).unwrap(),
            root: FixedBytes::from_hex(mmr_proof.root).unwrap(),
            elements_index: mmr_proof.elements_index,
            elements_count: mmr_proof.elements_count,
            path: mmr_proof
                .path
                .iter()
                .map(|hash| FixedBytes::from_hex(hash).unwrap())
                .collect(),
            peaks: mmr_proof
                .peaks
                .iter()
                .map(|hash| FixedBytes::from_hex(hash).unwrap())
                .collect(),
        }
    }
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct MmrProof {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunction,
    pub header_hash: FixedBytes<32>,
    pub root: FixedBytes<32>,
    pub elements_index: u64,
    pub elements_count: u64,
    pub path: Vec<FixedBytes<32>>,
    pub peaks: Vec<FixedBytes<32>>,
}
