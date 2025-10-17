//! Bankai block proof verification
//!
//! This module provides the core verification functions for Bankai's stateless light client:
//! - STWO zero-knowledge proof verification
//! - MMR (Merkle Mountain Range) inclusion proof verification

/// MMR (Merkle Mountain Range) proof verification
///
/// Functions for verifying that headers are committed in MMRs using inclusion proofs.
pub mod mmr;

/// STWO zero-knowledge proof verification
///
/// Functions for verifying STWO proofs and extracting trusted Bankai blocks with MMR roots.
pub mod stwo;
