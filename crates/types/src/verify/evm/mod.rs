extern crate alloc;
use alloc::vec::Vec;

pub mod beacon;
pub mod execution;

use alloy_consensus::TxEnvelope;
use alloy_primitives::U256;

use crate::verify::evm::beacon::BeaconHeader;
use crate::verify::evm::execution::Account;
use crate::verify::evm::execution::ExecutionHeader;

#[cfg_attr(feature = "std", derive(Debug))]
pub struct EvmResults {
    pub execution_header: Vec<ExecutionHeader>,
    pub beacon_header: Vec<BeaconHeader>,
    pub account: Vec<Account>,
    /// Each entry contains verified (slot_key, slot_value) pairs from a StorageSlotProof
    pub storage_slot: Vec<Vec<(U256, U256)>>,
    pub tx: Vec<TxEnvelope>,
}
