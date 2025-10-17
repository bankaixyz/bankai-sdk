//! Bankai Verify - Trustless Verification Library
//!
//! # Overview
//!
//! This library provides trustless verification of blockchain data using Bankai Block proofs.
//! It works in tandem with the `bankai-sdk` crate, which fetches proofs that this library verifies.
//!
//! **Once data is verified through this library, it is cryptographically guaranteed to be valid.
//! No further checks are needed.** This guarantee is provided by Bankai's stateless light client
//! architecture, which uses zero-knowledge proofs to establish an unbreakable chain of trust from
//! the STWO proof down to individual blockchain data elements.
//!
//! ## How Verification Works
//!
//! The verification process follows a hierarchical trust chain:
//!
//! 1. **Verify STWO Block Proof**: First, verify the STWO zero-knowledge proof to establish
//!    trust in the MMR roots it contains. This proof is cryptographically sound and cannot be forged.
//! 2. **Verify MMR Proofs**: Use the trusted MMR roots to verify individual header commitments
//!    through MMR inclusion proofs. Headers are guaranteed to be in the MMR.
//! 3. **Verify Chain Data**: Once headers are verified, use standard Merkle proofs to verify
//!    accounts, transactions, and other chain data against the header roots. This establishes
//!    that the data existed in that specific block.
//!
//! ## Stateless Light Client Architecture
//!
//! Bankai's architecture enables **stateless verification**: you don't need to maintain any state,
//! sync any chains, or trust any intermediaries. Each proof bundle is completely self-contained
//! and can be verified independently. This makes it perfect for:
//!
//! - Cross-chain bridges that need to verify data from other chains
//! - Smart contracts that need trustless access to historical blockchain data
//! - Applications that need blockchain data without running full nodes
//! - Zero-knowledge circuits that need verified inputs
//!

//! ## Usage
//!
//! ### Batch Verification (Recommended)
//!
//! The simplest way to verify proofs is using the batch verification function:
//!
//! ```no_run
//! use bankai_verify::verify_batch_proof;
//! use bankai_types::fetch::ProofWrapper;
//!
//! # fn example(proof_wrapper: ProofWrapper) -> Result<(), Box<dyn std::error::Error>> {
//! // Verify an entire batch of proofs at once
//! let results = verify_batch_proof(&proof_wrapper)?;
//!
//! // Access verified data
//! for header in &results.evm.execution_header {
//!     println!("Verified execution header at block {}", header.number);
//! }
//!
//! for account in &results.evm.account {
//!     println!("Verified account with balance: {}", account.balance);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Verify Block Proof Only
//!
//! If you only need the verified Bankai block (with MMR roots), you can verify just the STWO proof:
//!
//! ```no_run
//! use bankai_verify::bankai::stwo::verify_stwo_proof;
//! use cairo_air::CairoProof;
//! use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
//!
//! # fn example(block_proof: CairoProof<Blake2sMerkleHasher>) -> Result<(), Box<dyn std::error::Error>> {
//! // Verify the STWO proof and extract the Bankai block
//! let bankai_block = verify_stwo_proof(&block_proof)?;
//!
//! // Access the verified MMR roots
//! println!("Execution MMR root (Keccak): {:?}", bankai_block.execution.mmr_root_keccak);
//! println!("Beacon MMR root (Keccak): {:?}", bankai_block.beacon.mmr_root_keccak);
//! println!("Block number: {}", bankai_block.block_number);
//! # Ok(())
//! # }
//! ```
//!
//! ### Verify MMR Proofs
//!
//! Once you have verified MMR roots from a Bankai block, you can verify MMR inclusion proofs:
//!
//! ```no_run
//! use bankai_verify::bankai::mmr::verify_mmr_proof;
//! use bankai_types::proofs::{MmrProofDto, HashingFunctionDto};
//! use alloy_primitives::FixedBytes;
//!
//! # fn example(
//! #     mmr_proof: MmrProofDto,
//! #     trusted_mmr_root: FixedBytes<32>
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! // Verify that a header is committed in the MMR
//! let header_hash = verify_mmr_proof(&mmr_proof, trusted_mmr_root)?;
//!
//! println!("Verified header hash: {:?}", header_hash);
//! # Ok(())
//! # }
//! ```
//!
//! ### Verify Header Proofs
//!
//! With a verified MMR root, you can verify individual header proofs:
//!
//! ```no_run
//! use bankai_verify::evm::{ExecutionVerifier, BeaconVerifier};
//! use bankai_types::fetch::evm::execution::ExecutionHeaderProof;
//! use alloy_primitives::FixedBytes;
//!
//! # fn example(
//! #     proof: ExecutionHeaderProof,
//! #     mmr_root: FixedBytes<32>
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! // Verify an execution header against a trusted MMR root
//! let verified_header = ExecutionVerifier::verify_header_proof(&proof, mmr_root)?;
//! println!("Header number: {}", verified_header.number);
//! println!("State root: {:?}", verified_header.state_root);
//!
//! // Now you can verify accounts against this header's state root
//! # Ok(())
//! # }
//! ```
//!
//! ### Verify Account and Transaction Proofs
//!
//! Once you have a verified header, you can verify account and transaction proofs against it:
//!
//! ```no_run
//! use bankai_verify::evm::ExecutionVerifier;
//! use bankai_types::fetch::evm::execution::{AccountProof, TxProof};
//! use alloy_rpc_types_eth::Header;
//!
//! # fn example(
//! #     account_proof: AccountProof,
//! #     tx_proof: TxProof,
//! #     verified_header: Header
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! // Verify an account exists in the header's state
//! let account = ExecutionVerifier::verify_account_proof(&account_proof, &verified_header)?;
//! println!("Account balance: {}", account.balance);
//!
//! // Verify a transaction is included in the block
//! let transaction = ExecutionVerifier::verify_tx_proof(&tx_proof, &verified_header)?;
//! println!("Transaction hash: {:?}", transaction.hash);
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **STWO Proof Verification**: Verifies zero-knowledge proofs generated by the STWO prover
//! - **MMR Proof Verification**: Verifies Merkle Mountain Range inclusion proofs
//! - **EVM Data Verification**: Verifies execution headers, beacon headers, accounts, and transactions
//! - **Batch Operations**: Efficiently verify multiple proofs in a single operation
//! - **No-Std Ready**: Can be compiled for no-std environments (SP1 compatibility planned)

// TODO: Enable for SP1 once async/await is resolved
// #![no_std]
// extern crate alloc;

// Keep batch module private
mod batch;

/// Bankai block proof verification
///
/// This module provides functions for verifying STWO zero-knowledge proofs and MMR inclusion proofs.
/// These are the foundational verification operations that establish trust in the system.
///
/// - [`bankai::stwo`] - Verify STWO zero-knowledge proofs to extract trusted Bankai blocks with MMR roots
/// - [`bankai::mmr`] - Verify MMR inclusion proofs against trusted MMR roots
pub mod bankai;

/// EVM-specific verification components
///
/// This module provides verifiers for individual EVM chain data proofs.
/// After verification, all returned data is cryptographically guaranteed valid.
pub mod evm;

// ============================================================================
// Public API
// ============================================================================

/// Batch proof verification
///
/// The main entry point for verifying complete proof batches generated by the SDK.
pub use crate::batch::verify_batch_proof;

// Re-export common types from bankai_types for convenience
pub use bankai_types::verify::{BatchResults, evm::EvmResults};

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during proof verification
///
/// All verification failures return a specific error indicating what validation failed.
/// These errors are designed to be informative for debugging while maintaining security.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VerifyError {
    /// The STWO zero-knowledge proof is invalid or malformed
    InvalidStwoProof,
    
    /// An MMR inclusion proof failed verification
    InvalidMmrProof,
    
    /// The MMR tree structure is invalid
    InvalidMmrTree,
    
    /// The MMR root in the proof doesn't match the expected root from the STWO proof
    InvalidMmrRoot,
    
    /// The header hash doesn't match the committed value in the MMR
    InvalidHeaderHash,
    
    /// A transaction Merkle proof failed verification against the header's transactions root
    InvalidTxProof,
    
    /// An account Merkle proof failed verification against the header's state root
    InvalidAccountProof,
    
    /// Referenced execution header not found in the verified headers list
    InvalidExecutionHeaderProof,
    
    /// The state root in the account proof doesn't match the header's state root
    InvalidStateRoot,
    
    /// A Merkle Patricia Trie proof verification failed
    InvalidMptProof,
    
    /// Failed to decode RLP-encoded data
    InvalidRlpDecode,
}

impl core::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidStwoProof => write!(f, "Invalid STWO proof"),
            Self::InvalidMmrProof => write!(f, "Invalid MMR proof"),
            Self::InvalidMmrTree => write!(f, "Invalid MMR tree"),
            Self::InvalidMmrRoot => write!(f, "Invalid MMR root"),
            Self::InvalidHeaderHash => write!(f, "Invalid header hash"),
            Self::InvalidTxProof => write!(f, "Invalid transaction proof"),
            Self::InvalidAccountProof => write!(f, "Invalid account proof"),
            Self::InvalidExecutionHeaderProof => write!(f, "Invalid execution header proof"),
            Self::InvalidStateRoot => write!(f, "Invalid state root"),
            Self::InvalidMptProof => write!(f, "Invalid MPT proof"),
            Self::InvalidRlpDecode => write!(f, "Invalid RLP decode"),
        }
    }
}

impl std::error::Error for VerifyError {}
