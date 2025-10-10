use crate::errors::{SdkError, SdkResult};
use bankai_types::api::proofs::HashingFunctionDto;
use bankai_types::fetch::evm::beacon::{BeaconHeader, BeaconHeaderProof};
use tree_hash::TreeHash;

use crate::verify::bankai::mmr::BankaiMmr;
use crate::verify::bankai::stwo::verify_stwo_proof;
use alloy_primitives::hex::ToHexExt;

pub struct BeaconVerifier;

impl BeaconVerifier {
    pub async fn verify_header_proof(proof: &BeaconHeaderProof) -> SdkResult<BeaconHeader> {
        let bankai_block = verify_stwo_proof(&proof.block_proof)
            .map_err(|e| SdkError::Verification(format!("stwo verification failed: {e}")))?;

        // Check the bankai block mmr root matches the mmr proof root
        match proof.mmr_proof.hashing_function {
            HashingFunctionDto::Keccak => {
                let expected = format!("0x{}", bankai_block.beacon.mmr_root_keccak.encode_hex());
                if proof.mmr_proof.root != expected {
                    return Err(SdkError::Verification("mmr root mismatch (keccak)".into()));
                }
            }
            HashingFunctionDto::Poseidon => {
                let expected = format!("0x{}", bankai_block.beacon.mmr_root_poseidon.encode_hex());
                if proof.mmr_proof.root != expected {
                    return Err(SdkError::Verification(
                        "mmr root mismatch (poseidon)".into(),
                    ));
                }
            }
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
