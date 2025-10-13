// TODO: Enable for SP1 once async/await is resolved
// #![no_std]
// extern crate alloc;

pub mod bankai;
pub mod batch;
pub mod evm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VerifyError {
    InvalidStwoProof,
    InvalidMmrProof,
    InvalidMmrTree,
    InvalidMmrRoot,
    InvalidHeaderHash,
    InvalidTxProof,
    InvalidAccountProof,
    InvalidExecutionHeaderProof,
    InvalidStateRoot,
    InvalidMptProof,
    InvalidRlpDecode,
}

impl core::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidStwoProof => write!(f, "Invalid Stwo proof"),
            Self::InvalidMmrProof => write!(f, "Invalid MMR proof"),
            Self::InvalidMmrTree => write!(f, "Invalid MMR tree"),
            Self::InvalidMmrRoot => write!(f, "Invalid MMR root"),
            Self::InvalidHeaderHash => write!(f, "Invalid header hash"),
            Self::InvalidTxProof => write!(f, "Invalid transaction proof"),
            Self::InvalidAccountProof => write!(f, "Invalid account proof"),
            Self::InvalidExecutionHeaderProof => write!(f, "Invalid execution header proof"),
            Self::InvalidStateRoot => write!(f, "Invalid state root"),
            Self::InvalidMptProof => write!(f, "Invalid MPT proof"),
            Self::InvalidRlpDecode => write!(f, "Invalid RLP decode"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerifyError {}
