use crate::verify::evm::EvmResults;

pub mod evm;

#[derive(Debug)]
pub struct BatchResults {
    pub evm: EvmResults,
}
