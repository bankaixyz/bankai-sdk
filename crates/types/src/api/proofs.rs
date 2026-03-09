extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::common::HashingFunction;

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
    pub hashing_function: HashingFunction,
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
pub struct BankaiMmrProofDto {
    pub reference_block_number: u64,
    pub target_block_number: u64,
    pub hashing_function: HashingFunction,
    pub block_hash: String,
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
    pub hashing_function: HashingFunction,
    pub header_hash: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct BankaiBlockProofDto {
    pub block_number: u64,
    pub proof: BlockProofPayloadDto,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(tag = "format", content = "data", rename_all = "lowercase")
)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum BlockProofPayloadDto {
    Bin(String),
    Json(serde_json::Value),
}

pub type BlakeCairoProof =
    cairo_air::CairoProof<stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher>;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct LightClientProofDto {
    pub block_proof: BankaiBlockProofDto,
    pub mmr_proofs: Vec<MmrProofDto>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct BankaiBlockProofWithMmrDto {
    pub block_proof: BankaiBlockProofDto,
    pub mmr_proof: BankaiMmrProofDto,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct LightClientProofRequestDto {
    pub bankai_block_number: Option<u64>,
    pub hashing_function: HashingFunction,
    pub requested_headers: Vec<HeaderRequestDto>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct HeaderRequestDto {
    pub network_id: u64,
    pub header_hash: String,
}

