extern crate alloc;

use alloc::vec::Vec;

use alloy_consensus::{ReceiptEnvelope, TxEnvelope};
use alloy_primitives::{Address, FixedBytes, U256};

use crate::results::evm::beacon::BeaconHeader;
use crate::results::evm::execution::{ExecutionHeader, TrieAccount};

pub mod beacon;
pub mod execution;
pub mod op_stack;

/// Identifies a verified chain block by network and block number.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BlockRef {
    pub network_id: u64,
    pub block_number: u64,
}

/// Verified account result with the request identity preserved.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VerifiedAccount {
    pub block: BlockRef,
    pub address: Address,
    pub account: TrieAccount,
}

/// Verified storage slot result with the request identity preserved.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VerifiedStorageSlots {
    pub block: BlockRef,
    pub address: Address,
    pub slots: Vec<(U256, U256)>,
}

/// Verified transaction result with the request identity preserved.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VerifiedTransaction {
    pub block: BlockRef,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: u64,
    pub tx: TxEnvelope,
}

/// Verified receipt result with the request identity preserved.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VerifiedReceipt {
    pub block: BlockRef,
    pub tx_hash: FixedBytes<32>,
    pub tx_index: u64,
    pub receipt: ReceiptEnvelope,
}

/// Verified Ethereum data returned from batch verification.
#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct EvmResults {
    /// Verified execution headers.
    pub execution_header: Vec<ExecutionHeader>,
    /// Verified beacon headers.
    pub beacon_header: Vec<BeaconHeader>,
    /// Verified accounts with block and address identity.
    pub account: Vec<VerifiedAccount>,
    /// Verified storage slot values grouped by request with block and address identity.
    pub storage_slot: Vec<VerifiedStorageSlots>,
    /// Verified transactions with block and transaction identity.
    pub tx: Vec<VerifiedTransaction>,
    /// Verified receipts with block and transaction identity.
    pub receipt: Vec<VerifiedReceipt>,
}
