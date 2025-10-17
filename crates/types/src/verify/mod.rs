//! Verification result types
//!
//! This module contains types representing verified blockchain data.
//! Once verification succeeds, all data in these types is cryptographically
//! guaranteed to be valid - no further checks are needed.

use crate::verify::evm::EvmResults;

pub mod evm;

/// Results from batch proof verification
///
/// Contains all verified data from a successful batch verification.
/// **All data in this struct is cryptographically guaranteed valid** through
/// Bankai's stateless light client architecture.
///
/// # Fields
///
/// - `evm` - Verified EVM chain data (headers, accounts, transactions)
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BatchResults {
    /// Verified EVM data (execution and beacon headers, accounts, transactions)
    pub evm: EvmResults,
}
