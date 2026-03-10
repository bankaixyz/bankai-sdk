use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Invalid Merkle tree")]
    InvalidMerkleTree,
    #[error("Invalid Merkle proof")]
    InvalidMerkleProof,
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unsupported RPC response: {0}")]
    Unsupported(String),
    #[error("Invalid trie root")]
    InvalidTrieRoot,
    #[error("Invalid transaction proof")]
    InvalidTxProof,
    #[error("Invalid receipt proof")]
    InvalidReceiptProof,
}
