extern crate alloc;

use alloc::vec::Vec;

use crate::block::OpChainClient;

#[cfg_attr(feature = "std", derive(Debug, Default))]
pub struct OpStackResults {
    pub verified_snapshots: Vec<OpChainClient>,
}
