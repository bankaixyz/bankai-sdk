use bankai_types::block::BankaiBlock;
use cairo_air::{utils::get_verification_output, CairoProof};
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use crate::errors::SdkResult;

pub fn verify_stwo_proof(proof: &CairoProof<Blake2sMerkleHasher>) -> SdkResult<BankaiBlock> {
    let verification_output = get_verification_output(&proof.claim.public_data.public_memory);
    let block = BankaiBlock::from_verication_output(&verification_output);
    Ok(block)
}
