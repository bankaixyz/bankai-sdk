//! Core proof types for Bankai
//!
//! This module defines the fundamental proof types used throughout the Bankai ecosystem,
//! including MMR (Merkle Mountain Range) proofs and hashing function specifications.
//!
//! All types in this module work in `no_std` environments, making them suitable for
//! use in ZK circuits, smart contracts, and other constrained environments.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;

/// MMR (Merkle Mountain Range) proof for header inclusion
///
/// Proves that a specific blockchain header is committed in a verified MMR.
/// The MMR roots are established by STWO zero-knowledge proofs in Bankai blocks.
///
/// # Fields
///
/// - `network_id` - Identifies which chain (0 = beacon, 1 = execution)
/// - `block_number` - Block number of the header being proven
/// - `hashing_function` - Hash function used for MMR (Keccak or Poseidon)
/// - `header_hash` - Hash of the header being proven
/// - `root` - MMR root hash (must match the root in verified Bankai block)
/// - `elements_index` - Position of this element in the MMR
/// - `elements_count` - Total number of elements in the MMR
/// - `path` - Merkle path for verification (sibling hashes)
/// - `peaks` - MMR peak hashes for the current MMR state
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

/// Request for an MMR proof
///
/// Used to request a proof that a specific header is committed in the MMR.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct MmrProofRequestDto {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String, // 0x…32
}

/// Hashing function used for MMR construction
///
/// Different hash functions can be used depending on the target environment:
/// - **Keccak**: Standard Ethereum hash, efficient in EVM environments
/// - **Poseidon**: ZK-friendly hash, efficient in zero-knowledge circuits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub enum HashingFunctionDto {
    /// Keccak-256 hash function (Ethereum standard)
    Keccak,
    /// Poseidon hash function (ZK-friendly)
    Poseidon,
}

/// STWO zero-knowledge proof for a Bankai block
///
/// Contains the cryptographic proof that establishes trust in the MMR roots.
/// This is the foundation of Bankai's stateless light client architecture.
#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct BankaiBlockProofDto {
    /// Bankai block number this proof corresponds to
    pub block_number: u64,
    /// STWO proof data (JSON serialized)
    #[cfg_attr(feature = "utoipa", schema(value_type = Object))]
    pub proof: serde_json::Value,
}

/// Complete light client proof bundle
///
/// Contains everything needed for stateless verification:
/// - STWO proof establishing trust in MMR roots
/// - MMR proofs for specific headers
///
/// This is a self-contained proof that can be verified without any prior state.
#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct LightClientProofDto {
    /// The STWO block proof establishing MMR root trust
    pub block_proof: BankaiBlockProofDto,
    /// MMR inclusion proofs for requested headers
    pub mmr_proofs: Vec<MmrProofDto>,
}

/// Request for a complete light client proof
///
/// Used to request a self-contained proof bundle for specific headers.
#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct LightClientProofRequestDto {
    /// Which Bankai block to use (None = latest)
    pub bankai_block_number: Option<u64>,
    /// Hash function for MMR proofs
    pub hashing_function: HashingFunctionDto,
    /// Headers to prove
    pub requested_headers: Vec<HeaderRequestDto>,
}

/// Request for a specific header proof
#[cfg(feature = "api")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
pub struct HeaderRequestDto {
    /// Network ID (0 = beacon, 1 = execution)
    pub network_id: u64,
    /// Header hash to prove
    pub header_hash: String,
}
