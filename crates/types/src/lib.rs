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
//! - [`common`] - Shared enums used across API, SDK, and verification
//! - [`api`] - API request/response types (requires `std` and `api` feature)
//! - [`utils`] - Utility functions (MMR operations)
//! - [`block`] - Bankai block representations with beacon and execution client data
//! - [`inputs`] - Typed verification inputs assembled by the SDK
//! - [`results`] - Verified outputs returned by the verifier
//!
//! ## Feature Flags
//!
//! - `std` (default) - Enable standard library support
//! - `api` - Enable API types (requires `std`)
//! - `inputs` - Enable typed verifier input types
//! - `results` - Enable typed verification result types
//! - `serde` - Enable serde serialization support
//! - `utoipa` - Enable OpenAPI schema generation

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod common;

/// API request and response types
///
/// Types for interacting with the Bankai API, including block queries,
/// proof requests, and chain statistics.
///
/// Requires the `api` feature flag.
#[cfg(feature = "api")]
pub mod api;

// Re-export commonly used types for easier access
#[cfg(feature = "api")]
pub use cairo_air;

/// Utility functions for MMR operations
///
/// Provides helpers for working with Merkle Mountain Ranges,
/// including peak calculations and position utilities.
pub mod utils;

/// Bankai block representations
///
/// Defines the structure of Bankai blocks, which contain verified
/// beacon and execution chain data with their respective MMR roots.
pub mod block;

#[cfg(feature = "inputs")]
pub mod inputs;

#[cfg(feature = "results")]
pub mod results;

#[cfg(feature = "inputs")]
pub use inputs::ProofBundle;
