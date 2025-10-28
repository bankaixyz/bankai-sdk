use bankai_types::block::BankaiBlock;
use cairo_air::{CairoProof, PreProcessedTraceVariant, utils::get_verification_output};
use stwo::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};

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
    let block = BankaiBlock::from_verication_output(&verification_output);
    Ok(block)
}
