use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrProofDto {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String,
    pub root: String,
    pub elements_index: u64,
    pub elements_count: u64,
    pub path: Vec<String>,
    pub peaks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrProofRequestDto {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String, // 0x…32
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HashingFunctionDto {
    Keccak,
    Poseidon,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BankaiBlockProofDto {
    pub block_number: u64,
    #[schema(value_type = Object)]
    pub proof: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LightClientProofDto {
    pub block_proof: BankaiBlockProofDto,
    pub mmr_proofs: Vec<MmrProofDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LightClientProofRequestDto {
    pub bankai_block_number: Option<u64>,
    pub hashing_function: HashingFunctionDto,
    pub requested_headers: Vec<HeaderRequestDto>
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HeaderRequestDto {
    pub network_id: u64,
    pub header_hash: String,
}