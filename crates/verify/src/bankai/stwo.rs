use bankai_types::block::{BankaiBlock, BankaiBlockHashOutput};
pub use cairo_air::{utils::get_verification_output, CairoProof, PreProcessedTraceVariant};
pub use stwo::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};

use crate::VerifyError;

pub fn verify_stwo_proof(
    proof: CairoProof<Blake2sMerkleHasher>,
) -> Result<BankaiBlockHashOutput, VerifyError> {
    let verification_output = get_verification_output(&proof.claim.public_data.public_memory);
    let result = cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(
        proof,
        PreProcessedTraceVariant::CanonicalWithoutPedersen,
    );
    if result.is_err() {
        return Err(VerifyError::InvalidStwoProof);
    }
    BankaiBlockHashOutput::from_verification_output(&verification_output)
        .ok_or(VerifyError::InvalidStwoProof)
}

pub fn verify_block_proof(
    proof: CairoProof<Blake2sMerkleHasher>,
    block: &BankaiBlock,
) -> Result<BankaiBlock, VerifyError> {
    let hash_output = verify_stwo_proof(proof)?;
    let expected_hash = block.compute_block_hash_keccak();
    if hash_output.block_hash != expected_hash {
        return Err(VerifyError::InvalidBlockHash);
    }
    Ok(block.clone())
}
