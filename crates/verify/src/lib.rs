//! Bankai verification for proof bundles fetched by `bankai-sdk`.
//!
//! In the normal flow, you fetch a [`bankai_types::inputs::ProofBundle`] with `bankai-sdk`,
//! then call [`verify_batch_proof`] to get verified results.
//!
//! ```no_run
//! use bankai_types::inputs::ProofBundle;
//! use bankai_verify::verify_batch_proof;
//!
//! # fn example(proof_bundle: ProofBundle) -> Result<(), Box<dyn std::error::Error>> {
//! let results = verify_batch_proof(proof_bundle)?;
//! println!("Verified {} execution headers", results.evm.execution_header.len());
//! println!("Verified {} OP Stack headers", results.op_stack.header.len());
//! # Ok(())
//! # }
//! ```
//!
//! Guides:
//!
//! - [Verify a proof](https://github.com/bankaixyz/bankai-docs/blob/main/content/docs/sdk/verify-a-proof.mdx)
//! - [Trust model](https://github.com/bankaixyz/bankai-docs/blob/main/content/docs/concepts/trust-model.mdx)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

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

/// Verifies a proof bundle returned by `bankai-sdk` and returns trusted results.
pub use crate::batch::verify_batch_proof;

// Re-export common types from bankai_types for convenience
pub use bankai_types::results::{evm::EvmResults, BatchResults};

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

    /// The provided full block witness does not match the trusted block hash
    InvalidBlockHash,

    /// An MMR inclusion proof failed verification
    InvalidMmrProof,

    /// The MMR tree structure is invalid
    InvalidMmrTree,

    /// The MMR root in the proof doesn't match the expected root from the STWO proof
    InvalidMmrRoot,

    /// A Merkle proof failed verification
    InvalidMerkleProof,

    /// The header hash doesn't match the committed value in the MMR
    InvalidHeaderHash,

    /// A transaction Merkle proof failed verification against the header's transactions root
    InvalidTxProof,

    /// A receipt Merkle proof failed verification against the header's receipts root
    InvalidReceiptProof,

    /// An account Merkle proof failed verification against the header's state root
    InvalidAccountProof,

    /// A storage Merkle proof failed verification against the account's storage root
    InvalidStorageProof,

    /// Referenced execution header not found in the verified headers list
    InvalidExecutionHeaderProof,

    /// The state root in the account proof doesn't match the header's state root
    InvalidStateRoot,

    /// Failed to decode RLP-encoded data
    InvalidRlpDecode,
}

impl core::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidStwoProof => write!(f, "Invalid STWO proof"),
            Self::InvalidBlockHash => write!(f, "Invalid block hash"),
            Self::InvalidMmrProof => write!(f, "Invalid MMR proof"),
            Self::InvalidMmrTree => write!(f, "Invalid MMR tree"),
            Self::InvalidMmrRoot => write!(f, "Invalid MMR root"),
            Self::InvalidMerkleProof => write!(f, "Invalid Merkle proof"),
            Self::InvalidHeaderHash => write!(f, "Invalid header hash"),
            Self::InvalidTxProof => write!(f, "Invalid transaction proof"),
            Self::InvalidReceiptProof => write!(f, "Invalid receipt proof"),
            Self::InvalidAccountProof => write!(f, "Invalid account proof"),
            Self::InvalidStorageProof => write!(f, "Invalid storage proof"),
            Self::InvalidExecutionHeaderProof => write!(f, "Invalid execution header proof"),
            Self::InvalidStateRoot => write!(f, "Invalid state root"),
            Self::InvalidRlpDecode => write!(f, "Invalid RLP decode"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerifyError {}
