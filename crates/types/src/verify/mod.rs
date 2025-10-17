use crate::verify::evm::EvmResults;

pub mod evm;

#[cfg_attr(feature = "std", derive(Debug))]
pub struct BatchResults {
    pub evm: EvmResults,
}
