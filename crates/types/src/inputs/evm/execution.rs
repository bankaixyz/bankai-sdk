extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use alloy_rpc_types_eth::{Account, Header as ExecutionHeader};
use serde::{Deserialize, Serialize};

use crate::inputs::evm::header_serde::{deserialize_execution_header, serialize_execution_header};
use crate::inputs::evm::MmrProof;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ExecutionHeaderProof {
    #[serde(
        serialize_with = "serialize_execution_header",
        deserialize_with = "deserialize_execution_header"
    )]
    pub header: ExecutionHeader,
    pub mmr_proof: MmrProof,
}

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

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiptProof {
    pub network_id: u64,
    pub block_number: u64,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: u64,
    pub proof: Vec<Bytes>,
    pub encoded_receipt: Vec<u8>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct StorageSlotEntry {
    pub slot_key: U256,
    pub slot_value: U256,
    pub storage_mpt_proof: Vec<Bytes>,
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct StorageSlotProof {
    pub account: Account,
    pub address: Address,
    pub network_id: u64,
    pub block_number: u64,
    pub state_root: FixedBytes<32>,
    pub account_mpt_proof: Vec<Bytes>,
    pub slots: Vec<StorageSlotEntry>,
}
