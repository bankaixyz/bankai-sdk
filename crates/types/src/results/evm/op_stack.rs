extern crate alloc;

use alloc::vec::Vec;

use alloy_consensus::{ReceiptEnvelope, TxEnvelope};
use alloy_primitives::U256;

use crate::results::evm::execution::{Account, ExecutionHeader};

/// Verified OP Stack data returned from batch verification.
#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct OpStackResults {
    /// Verified OP Stack headers.
    pub header: Vec<ExecutionHeader>,
    /// Verified OP Stack accounts.
    pub account: Vec<Account>,
    /// Verified OP Stack storage slot values grouped by request.
    pub storage_slot: Vec<Vec<(U256, U256)>>,
    /// Verified OP Stack transactions.
    pub tx: Vec<TxEnvelope>,
    /// Verified OP Stack receipts.
    pub receipt: Vec<ReceiptEnvelope>,
}
