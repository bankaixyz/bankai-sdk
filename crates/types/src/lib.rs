//! # Bankai Types
//!
//! Shared type definitions for the Bankai SDK ecosystem.
//!
//! This crate provides common types used across the Bankai SDK, verification library,
//! and API. It's designed to work in both `std` and `no_std` environments, making it
//! suitable for use in constrained environments like smart contracts and ZK circuits.
//!
//! ## Modules
//!
//! - [`proofs`] - Core proof types (MMR proofs, hashing functions) - works in `no_std`
//! - [`api`] - API request/response types (requires `std` and `api` feature)
//! - [`utils`] - Utility functions (MMR operations)
//! - [`block`] - Bankai block representations with beacon and execution client data
//! - [`fetch`] - Types for proof fetching and wrapping (requires `verifier-types` feature)
//! - [`verify`] - Types for verification results (requires `verifier-types` feature)
//!
//! ## Feature Flags
//!
//! - `std` (default) - Enable standard library support
//! - `api` - Enable API types (requires `std`)
//! - `verifier-types` - Enable verifier-specific types (fetch/verify modules)
//! - `serde` - Enable serde serialization support
//! - `utoipa` - Enable OpenAPI schema generation

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// Core proof types for MMR proofs and hashing functions
///
/// These types are available in both `std` and `no_std` environments,
/// making them suitable for use in ZK circuits and smart contracts.
pub mod proofs;

/// API request and response types
///
/// Types for interacting with the Bankai API, including block queries,
/// proof requests, and chain statistics.
///
/// Requires the `api` feature flag.
#[cfg(any(feature = "default", feature = "api"))]
pub mod api;

/// Utility functions for MMR operations
///
/// Provides helpers for working with Merkle Mountain Ranges,
/// including peak calculations and position utilities.
pub mod utils;

/// Bankai block representations
///
/// Defines the structure of Bankai blocks, which contain verified
/// beacon and execution chain data with their respective MMR roots.
#[cfg(any(feature = "default", feature = "api", feature = "verifier-types"))]
pub mod block;

/// Proof fetching types
///
/// Types used for fetching and wrapping proofs from the Bankai API,
/// including EVM-specific proof structures. The main type is [`fetch::ProofBundle`],
/// which bundles together all proofs needed for batch verification.
///
/// Requires the `verifier-types` feature flag.
#[cfg(feature = "verifier-types")]
pub mod fetch;

/// Verification result types
///
/// Types representing verified data after successful proof verification,
/// including batch results and EVM-specific verified data.
///
/// Requires the `verifier-types` feature flag.
#[cfg(feature = "verifier-types")]
pub mod verify;

// Re-export commonly used types for easier access
#[cfg(feature = "verifier-types")]
pub use fetch::ProofBundle;
