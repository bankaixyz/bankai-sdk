//! Verified outputs returned by the verification crate.

use crate::results::evm::{op_stack::OpStackResults, EvmResults};

pub mod evm;

/// Verified results returned by [`bankai_verify::verify_batch_proof`].
#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct BatchResults {
    /// Verified Ethereum execution, beacon, and state data.
    pub evm: EvmResults,
    /// Verified OP Stack headers and state data.
    pub op_stack: OpStackResults,
}
