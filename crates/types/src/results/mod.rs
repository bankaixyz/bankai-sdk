//! Verified outputs returned by the verification crate.

use crate::results::{evm::EvmResults, op_stack::OpStackResults};

pub mod evm;
pub mod op_stack;

#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct BatchResults {
    pub evm: EvmResults,
    pub op_stack: OpStackResults,
}
