use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::ethereum::BankaiBlockFilterDto;
use crate::api::proofs::{BankaiBlockProofDto, MmrProofDto};
use crate::block::OpChainClient;
use crate::common::{HashingFunction, ProofFormat};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpChainsSummaryDto {
    pub n_clients: u64,
    pub chains: Vec<OpChainSnapshotSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpChainSnapshotSummaryDto {
    pub chain_id: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub header_hash: String,
    pub l1_submission_block: u64,
    pub mmr_roots: super::blocks::MmrRootsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpMerkleProofDto {
    pub bankai_block_number: u64,
    pub chain_id: u64,
    pub merkle_leaf_index: u64,
    pub leaf_hash: String,
    pub root: String,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpStackMmrProofDto {
    pub merkle_proof: OpMerkleProofDto,
    pub mmr_proof: MmrProofDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpStackLightClientProofDto {
    pub block_proof: BankaiBlockProofDto,
    #[cfg_attr(feature = "utoipa", schema(value_type = Object))]
    pub snapshot: OpChainClient,
    pub merkle_proof: OpMerkleProofDto,
    pub mmr_proofs: Vec<MmrProofDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpStackMerkleProofRequestDto {
    pub filter: BankaiBlockFilterDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpStackMmrProofRequestDto {
    pub filter: BankaiBlockFilterDto,
    pub hashing_function: HashingFunction,
    pub header_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpStackLightClientProofRequestDto {
    pub filter: BankaiBlockFilterDto,
    pub hashing_function: HashingFunction,
    pub header_hashes: Vec<String>,
    #[serde(default)]
    pub proof_format: ProofFormat,
}
