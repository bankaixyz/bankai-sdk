extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct MmrProofRequestDto {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String, // 0x…32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum HashingFunctionDto {
    Keccak,
    Poseidon,
}

#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct BankaiBlockProofDto {
    pub block_number: u64,
    #[cfg_attr(feature = "utoipa", schema(value_type = Object))]
    pub proof: serde_json::Value,
}

#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct LightClientProofDto {
    pub block_proof: BankaiBlockProofDto,
    pub mmr_proofs: Vec<MmrProofDto>,
}

#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct LightClientProofRequestDto {
    pub bankai_block_number: Option<u64>,
    pub hashing_function: HashingFunctionDto,
    pub requested_headers: Vec<HeaderRequestDto>,
}

#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct HeaderRequestDto {
    pub network_id: u64,
    pub header_hash: String,
}
