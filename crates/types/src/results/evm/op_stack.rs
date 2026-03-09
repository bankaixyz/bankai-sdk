extern crate alloc;

use alloc::vec::Vec;

use alloy_consensus::{ReceiptEnvelope, TxEnvelope};
use alloy_primitives::U256;

use crate::results::evm::execution::{Account, ExecutionHeader};

#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct OpStackResults {
    pub header: Vec<ExecutionHeader>,
    pub account: Vec<Account>,
    pub storage_slot: Vec<Vec<(U256, U256)>>,
    pub tx: Vec<TxEnvelope>,
    pub receipt: Vec<ReceiptEnvelope>,
}
