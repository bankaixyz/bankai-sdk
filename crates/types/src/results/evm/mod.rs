extern crate alloc;

use alloc::vec::Vec;

use alloy_consensus::TxEnvelope;
use alloy_primitives::U256;

use crate::results::evm::beacon::BeaconHeader;
use crate::results::evm::execution::{Account, ExecutionHeader};

pub mod beacon;
pub mod execution;

#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct EvmResults {
    pub execution_header: Vec<ExecutionHeader>,
    pub beacon_header: Vec<BeaconHeader>,
    pub account: Vec<Account>,
    pub storage_slot: Vec<Vec<(U256, U256)>>,
    pub tx: Vec<TxEnvelope>,
}
