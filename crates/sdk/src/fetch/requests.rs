use alloy_primitives::{Address, FixedBytes, U256};

#[derive(Debug, Default)]
pub struct EvmProofsRequest {
    pub execution_header: Option<Vec<ExecutionHeaderProofRequest>>,
    pub beacon_header: Option<Vec<BeaconHeaderProofRequest>>,
    pub account: Option<Vec<AccountProofRequest>>,
    pub storage_slot: Option<Vec<StorageSlotProofRequest>>,
    pub tx_proof: Option<Vec<TxProofRequest>>,
    pub receipt_proof: Option<Vec<ReceiptProofRequest>>,
}

#[derive(Debug, Default)]
pub struct OpStackProofsRequest {
    pub header: Vec<OpStackHeaderProofRequest>,
    pub account: Vec<OpStackAccountProofRequest>,
    pub storage_slot: Vec<OpStackStorageSlotProofRequest>,
    pub tx_proof: Vec<OpStackTxProofRequest>,
    pub receipt_proof: Vec<OpStackReceiptProofRequest>,
}

#[derive(Debug)]
pub struct ExecutionHeaderProofRequest {
    pub network_id: u64,
    pub block_number: u64,
}

#[derive(Debug)]
pub struct BeaconHeaderProofRequest {
    pub network_id: u64,
    pub slot: u64,
}

#[derive(Debug)]
pub struct AccountProofRequest {
    pub network_id: u64,
    pub block_number: u64,
    pub address: Address,
}

#[derive(Debug)]
pub struct StorageSlotProofRequest {
    pub network_id: u64,
    pub block_number: u64,
    pub address: Address,
    pub slot_keys: Vec<U256>,
}

#[derive(Debug)]
pub struct TxProofRequest {
    pub network_id: u64,
    pub tx_hash: FixedBytes<32>,
}

#[derive(Debug)]
pub struct ReceiptProofRequest {
    pub network_id: u64,
    pub tx_hash: FixedBytes<32>,
}

#[derive(Debug)]
pub struct OpStackHeaderProofRequest {
    pub chain_name: String,
    pub block_number: Option<u64>,
    pub header_hash: Option<FixedBytes<32>>,
}

#[derive(Debug)]
pub struct OpStackAccountProofRequest {
    pub chain_name: String,
    pub block_number: u64,
    pub address: Address,
}

#[derive(Debug)]
pub struct OpStackStorageSlotProofRequest {
    pub chain_name: String,
    pub block_number: u64,
    pub address: Address,
    pub slot_keys: Vec<U256>,
}

#[derive(Debug)]
pub struct OpStackTxProofRequest {
    pub chain_name: String,
    pub tx_hash: FixedBytes<32>,
}

#[derive(Debug)]
pub struct OpStackReceiptProofRequest {
    pub chain_name: String,
    pub tx_hash: FixedBytes<32>,
}
