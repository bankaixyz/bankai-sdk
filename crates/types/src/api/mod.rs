//! API request and response types
//!
//! This module contains all types used for interacting with the Bankai API.
//! These types represent the JSON structures sent to and received from API endpoints.

/// Block query types and responses
pub mod blocks;

/// Chain configuration and metadata
pub mod chains;

/// Explorer-oriented aggregate API types
pub mod explorer;

/// API error types
pub mod error;

/// Proof request and response types
pub mod proofs;

/// Ethereum light-client API types
pub mod ethereum;

/// OP Stack proof request and response types
pub mod op_stack;

/// Chain statistics and metrics
pub mod stats;
