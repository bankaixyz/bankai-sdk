extern crate alloc;

use alloc::vec::Vec;

use alloy_consensus::{ReceiptEnvelope, TxEnvelope};
use alloy_primitives::U256;

use crate::results::evm::beacon::BeaconHeader;
use crate::results::evm::execution::{ExecutionHeader, TrieAccount};

pub mod beacon;
pub mod execution;
pub mod op_stack;

/// Verified Ethereum data returned from batch verification.
#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct EvmResults {
    /// Verified execution headers.
    pub execution_header: Vec<ExecutionHeader>,
    /// Verified beacon headers.
    pub beacon_header: Vec<BeaconHeader>,
    /// Verified accounts.
    pub account: Vec<TrieAccount>,
    /// Verified storage slot values grouped by request.
    pub storage_slot: Vec<Vec<(U256, U256)>>,
    /// Verified transactions.
    pub tx: Vec<TxEnvelope>,
    /// Verified receipts.
    pub receipt: Vec<ReceiptEnvelope>,
}
