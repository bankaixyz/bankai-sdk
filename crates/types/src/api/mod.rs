//! API request and response types
//!
//! This module contains all types used for interacting with the Bankai API.
//! These types represent the JSON structures sent to and received from API endpoints.

/// Block query types and responses
pub mod blocks;

/// Chain configuration and metadata
pub mod chains;

/// API error types
pub mod error;

/// Proof request and response types (re-exports from root proofs module)
pub mod proofs;

/// Chain statistics and metrics
pub mod stats;
