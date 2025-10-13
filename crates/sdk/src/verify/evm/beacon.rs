use crate::errors::{SdkError, SdkResult};
use bankai_types::fetch::evm::beacon::BeaconHeaderProof;
use bankai_types::verify::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use crate::verify::bankai::mmr::BankaiMmr;
use alloy_primitives::hex::ToHexExt;

pub struct BeaconVerifier;

impl BeaconVerifier {
    pub async fn verify_header_proof(
        proof: &BeaconHeaderProof,
        root: String,
    ) -> SdkResult<BeaconHeader> {
        if proof.mmr_proof.root != root {
            return Err(SdkError::Verification(format!(
                "mmr root mismatch! {} != {}",
                proof.mmr_proof.root, root
            )));
        }

        // Verify the mmr proof
        let mmr_proof_valid = BankaiMmr::verify_mmr_proof(proof.mmr_proof.clone())
            .await
            .map_err(|e| SdkError::Verification(format!("mmr verify error: {e}")))?;
        if !mmr_proof_valid {
            return Err(SdkError::Verification("invalid mmr proof".into()));
        }

        // Check the header hash matches the mmr proof header hash
        let hash = proof.header.tree_hash_root();
        let expected_header_hash = format!("0x{}", hash.encode_hex());
        if expected_header_hash != proof.mmr_proof.header_hash {
            return Err(SdkError::Verification("header hash mismatch".into()));
        }

        Ok(proof.header.clone())
    }
}
