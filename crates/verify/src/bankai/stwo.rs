use bankai_types::block::{BankaiBlock, BankaiBlockHashOutput};
pub use cairo_air::{utils::get_verification_output, CairoProof, PreProcessedTraceVariant};
pub use stwo::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};

use crate::VerifyError;

pub fn verify_stwo_proof(
    proof: CairoProof<Blake2sMerkleHasher>,
) -> Result<BankaiBlock, VerifyError> {
    let verification_output = get_verification_output(&proof.claim.public_data.public_memory);
    let result = cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(
        proof,
        PreProcessedTraceVariant::CanonicalWithoutPedersen,
    );
    if result.is_err() {
        return Err(VerifyError::InvalidStwoProof);
    }
    BankaiBlock::from_verication_output(&verification_output).ok_or(VerifyError::InvalidStwoProof)
}

pub fn verify_stwo_proof_hash_output(
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
    BankaiBlockHashOutput::from_verication_output(&verification_output)
        .ok_or(VerifyError::InvalidStwoProof)
}
