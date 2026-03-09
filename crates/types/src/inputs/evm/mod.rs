extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::FixedBytes;
#[cfg(feature = "api")]
use alloy_primitives::hex::FromHex;
use serde::{Deserialize, Serialize};

#[cfg(feature = "api")]
use crate::api::proofs::MmrProofDto;
use crate::common::HashingFunction;
use crate::inputs::evm::{
    beacon::BeaconHeaderProof,
    execution::{AccountProof, ExecutionHeaderProof, ReceiptProof, StorageSlotProof, TxProof},
};

pub mod beacon;
pub mod execution;
pub(crate) mod header_serde;
pub mod op_stack;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Default, Serialize, Deserialize)]
pub struct EvmProofs {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_header_proof: Vec<ExecutionHeaderProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub beacon_header_proof: Vec<BeaconHeaderProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub account_proof: Vec<AccountProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub storage_slot_proof: Vec<StorageSlotProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tx_proof: Vec<TxProof>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipt_proof: Vec<ReceiptProof>,
}

impl EvmProofs {
    pub fn is_empty(&self) -> bool {
        self.execution_header_proof.is_empty()
            && self.beacon_header_proof.is_empty()
            && self.account_proof.is_empty()
            && self.storage_slot_proof.is_empty()
            && self.tx_proof.is_empty()
            && self.receipt_proof.is_empty()
    }
}

#[cfg(feature = "api")]
impl TryFrom<MmrProofDto> for MmrProof {
    type Error = alloy_primitives::hex::FromHexError;

    fn try_from(mmr_proof: MmrProofDto) -> Result<Self, Self::Error> {
        Ok(MmrProof {
            network_id: mmr_proof.network_id,
            block_number: mmr_proof.block_number,
            hashing_function: mmr_proof.hashing_function,
            header_hash: FixedBytes::from_hex(mmr_proof.header_hash)?,
            root: FixedBytes::from_hex(mmr_proof.root)?,
            elements_index: mmr_proof.elements_index,
            elements_count: mmr_proof.elements_count,
            path: mmr_proof
                .path
                .iter()
                .map(FixedBytes::from_hex)
                .collect::<Result<Vec<_>, _>>()?,
            peaks: mmr_proof
                .peaks
                .iter()
                .map(FixedBytes::from_hex)
                .collect::<Result<Vec<_>, _>>()?,
        })
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
