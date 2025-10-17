extern crate alloc;
use alloc::format;
use alloc::string::String;

use alloy_primitives::FixedBytes;
use bankai_types::fetch::evm::beacon::BeaconHeaderProof;
use bankai_types::verify::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use alloy_primitives::hex::ToHexExt;

use crate::bankai::mmr_new::CairoLikeMmr;
use crate::VerifyError;

pub struct BeaconVerifier;

impl BeaconVerifier {
    pub fn verify_header_proof(
        proof: &BeaconHeaderProof,
        root: FixedBytes<32>,
    ) -> Result<BeaconHeader, VerifyError> {
        if proof.mmr_proof.root != root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        CairoLikeMmr::verify_mmr_proof(&proof.mmr_proof.clone())?;

        let hash = proof.header.tree_hash_root();
        if hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone())
    }
}
