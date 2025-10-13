pub mod bankai;
pub mod batch;
pub mod evm;

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum VerifyError {
    #[error("Invalid Stwo proof")]
    InvalidStwoProof,
    #[error("Invalid MMR proof")]
    InvalidMmrProof,
    #[error("Invalid MMR tree")]
    InvalidMmrTree,
    #[error("Invalid MMR root")]
    InvalidMmrRoot,
    #[error("Invalid header hash")]
    InvalidHeaderHash,
    #[error("Invalid transaction proof")]
    InvalidTxProof,
    #[error("Invalid account proof")]
    InvalidAccountProof,
    #[error("Invalid execution header proof")]
    InvalidExecutionHeaderProof,
    #[error("Invalid state root")]
    InvalidStateRoot,
    #[error("Invalid MPT proof")]
    InvalidMptProof,
    #[error("Invalid RLP decode")]
    InvalidRlpDecode,
}
