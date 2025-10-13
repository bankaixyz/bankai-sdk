pub mod beacon;
pub mod execution;

use crate::verify::evm::beacon::BeaconHeader;
use crate::verify::evm::execution::Account;
use crate::verify::evm::execution::ExecutionHeader;

#[derive(Debug)]
pub struct EvmResults {
    pub execution_header: Vec<ExecutionHeader>,
    pub beacon_header: Vec<BeaconHeader>,
    pub account: Vec<Account>,
}
