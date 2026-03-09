use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Invalid Merkle tree")]
    InvalidMerkleTree,
    #[error("Invalid Merkle proof")]
    InvalidMerkleProof,
}
