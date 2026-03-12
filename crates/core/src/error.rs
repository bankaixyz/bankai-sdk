use alloc::string::String;
use core::fmt;

#[derive(Debug)]
pub enum CoreError {
    InvalidMerkleTree,
    InvalidMerkleProof,
    InvalidOpStackCommitment,
    Provider(String),
    NotFound(String),
    Unsupported(String),
    InvalidTrieRoot,
    InvalidTxProof,
    InvalidReceiptProof,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMerkleTree => write!(f, "Invalid Merkle tree"),
            Self::InvalidMerkleProof => write!(f, "Invalid Merkle proof"),
            Self::InvalidOpStackCommitment => write!(f, "Invalid OP Stack commitment"),
            Self::Provider(message) => write!(f, "Provider error: {message}"),
            Self::NotFound(message) => write!(f, "Not found: {message}"),
            Self::Unsupported(message) => write!(f, "Unsupported RPC response: {message}"),
            Self::InvalidTrieRoot => write!(f, "Invalid trie root"),
            Self::InvalidTxProof => write!(f, "Invalid transaction proof"),
            Self::InvalidReceiptProof => write!(f, "Invalid receipt proof"),
        }
    }
}
