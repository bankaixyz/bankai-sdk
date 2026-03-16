extern crate alloc;

use alloc::vec::Vec;

use crate::results::evm::{
    execution::ExecutionHeader, VerifiedAccount, VerifiedReceipt, VerifiedStorageSlots,
    VerifiedTransaction,
};

/// Verified OP Stack data returned from batch verification.
#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct OpStackResults {
    /// Verified OP Stack headers.
    pub header: Vec<ExecutionHeader>,
    /// Verified OP Stack accounts with block and address identity.
    pub account: Vec<VerifiedAccount>,
    /// Verified OP Stack storage slot values grouped by request with block and address identity.
    pub storage_slot: Vec<VerifiedStorageSlots>,
    /// Verified OP Stack transactions with block and transaction identity.
    pub tx: Vec<VerifiedTransaction>,
    /// Verified OP Stack receipts with block and transaction identity.
    pub receipt: Vec<VerifiedReceipt>,
}
