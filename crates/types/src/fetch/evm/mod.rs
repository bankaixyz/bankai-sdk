extern crate alloc;
use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, hex::FromHex};
use serde::{Deserialize, Serialize};

use crate::{
    fetch::evm::{
        beacon::BeaconHeaderProof,
        execution::{AccountProof, ExecutionHeaderProof, TxProof},
    },
    proofs::{HashingFunctionDto, MmrProofDto},
};

pub mod beacon;
pub mod execution;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct EvmProofs {
    pub execution_header_proof: Option<Vec<ExecutionHeaderProof>>,
    pub beacon_header_proof: Option<Vec<BeaconHeaderProof>>,
    pub account_proof: Option<Vec<AccountProof>>,
    pub tx_proof: Option<Vec<TxProof>>,
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
                .map(|h| FixedBytes::from_hex(h).unwrap())
                .collect(),
            peaks: mmr_proof
                .peaks
                .iter()
                .map(|h| FixedBytes::from_hex(h).unwrap())
                .collect(),
        }
    }
}

#[cfg(feature = "verifier-types")]
#[cfg_attr(any(feature = "verifier-types", feature = "std"), derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct MmrProof {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: FixedBytes<32>,
    pub root: FixedBytes<32>,
    pub elements_index: u64,
    pub elements_count: u64,
    pub path: Vec<FixedBytes<32>>,
    pub peaks: Vec<FixedBytes<32>>,
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
