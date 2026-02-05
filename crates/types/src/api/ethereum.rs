use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::proofs::HashingFunctionDto;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BankaiBlockSelectorDto {
    Latest,
    Justified,
    Finalized,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct BankaiBlockFilterDto {
    pub selector: Option<BankaiBlockSelectorDto>,
    pub bankai_block_number: Option<u64>,
}

impl BankaiBlockFilterDto {
    pub fn latest() -> Self {
        Self {
            selector: Some(BankaiBlockSelectorDto::Latest),
            bankai_block_number: None,
        }
    }

    pub fn justified() -> Self {
        Self {
            selector: Some(BankaiBlockSelectorDto::Justified),
            bankai_block_number: None,
        }
    }

    pub fn finalized() -> Self {
        Self {
            selector: Some(BankaiBlockSelectorDto::Finalized),
            bankai_block_number: None,
        }
    }

    pub fn with_bankai_block_number(bankai_block_number: u64) -> Self {
        Self {
            selector: None,
            bankai_block_number: Some(bankai_block_number),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HeightDto {
    pub height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrSnapshotDto {
    pub keccak_root: String,
    pub poseidon_root: String,
    pub elements_count: u64,
    pub leafs_count: u64,
    pub keccak_peaks: Vec<String>,
    pub poseidon_peaks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BeaconSnapshotDto {
    pub chain_id: u64,
    pub epoch_number: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub beacon_root: String,
    pub state_root: String,
    pub justified_height: u64,
    pub finalized_height: u64,
    pub mmr_snapshot: MmrSnapshotDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExecutionSnapshotDto {
    pub chain_id: u64,
    pub epoch_number: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub header_hash: String,
    pub justified_height: u64,
    pub finalized_height: u64,
    pub mmr_snapshot: MmrSnapshotDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EthereumEpochDto {
    pub number: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub num_signers: u64,
    pub epochs_count: u32,
    pub block_number: u64,
    pub sync_committee_term_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncCommitteeKeysDto {
    pub chain_id: u64,
    pub term_id: u64,
    pub pubkeys: Vec<String>,
    pub aggregate_pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EthereumMmrProofRequestDto {
    pub filter: BankaiBlockFilterDto,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EthereumLightClientProofRequestDto {
    pub filter: BankaiBlockFilterDto,
    pub hashing_function: HashingFunctionDto,
    pub header_hashes: Vec<String>,
}
