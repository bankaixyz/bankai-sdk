use alloy_primitives::{Address, FixedBytes};

use crate::{
    api::proofs::HashingFunctionDto,
    fetch::evm::{
        beacon::BeaconHeaderProof,
        execution::{AccountProof, ExecutionHeaderProof, TxProof},
    },
};

pub mod beacon;
pub mod execution;

#[derive(Debug)]
pub struct EvmProofs {
    pub execution_header_proof: Option<Vec<ExecutionHeaderProof>>,
    pub beacon_header_proof: Option<Vec<BeaconHeaderProof>>,
    pub account_proof: Option<Vec<AccountProof>>,
    pub tx_proof: Option<Vec<TxProof>>,
}

#[derive(Debug)]
pub struct EvmProofsRequest {
    pub execution_header: Option<Vec<ExecutionHeaderProofRequest>>,
    pub beacon_header: Option<Vec<BeaconHeaderProofRequest>>,
    pub account: Option<Vec<AccountProofRequest>>,
    pub tx_proof: Option<Vec<TxProofRequest>>,
}

#[derive(Debug)]
pub struct ExecutionHeaderProofRequest {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub bankai_block_number: u64,
}

#[derive(Debug)]
pub struct BeaconHeaderProofRequest {
    pub network_id: u64,
    pub slot: u64,
    pub hashing_function: HashingFunctionDto,
    pub bankai_block_number: u64,
}

#[derive(Debug)]
pub struct AccountProofRequest {
    pub network_id: u64,
    pub block_number: u64,
    pub address: Address,
}

#[derive(Debug)]
pub struct TxProofRequest {
    pub network_id: u64,
    pub tx_hash: FixedBytes<32>,
}
