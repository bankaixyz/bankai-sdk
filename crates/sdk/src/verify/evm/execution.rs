use bankai_types::api::proofs::HashingFunctionDto;
use bankai_types::fetch::evm::execution::ExecutionHeaderProof;

use crate::errors::{SdkError, SdkResult};
use alloy_rpc_types::Header as ExecutionHeader;

use crate::verify::bankai::mmr::BankaiMmr;
use crate::verify::bankai::stwo::verify_stwo_proof;
use alloy_primitives::hex::ToHexExt;

pub struct ExecutionVerifier;

impl ExecutionVerifier {
    pub async fn verify_header_proof(proof: &ExecutionHeaderProof) -> SdkResult<ExecutionHeader> {
        let bankai_block = verify_stwo_proof(&proof.block_proof)
            .map_err(|e| SdkError::Verification(format!("stwo verification failed: {e}")))?;

        // Check the bankai block mmr root matches the mmr proof root
        match proof.mmr_proof.hashing_function {
            HashingFunctionDto::Keccak => {
                let expected = format!("0x{}", bankai_block.execution.mmr_root_keccak.encode_hex());
                if proof.mmr_proof.root != expected {
                    return Err(SdkError::Verification("mmr root mismatch (keccak)".into()));
                }
            }
            HashingFunctionDto::Poseidon => {
                let expected = format!(
                    "0x{}",
                    bankai_block.execution.mmr_root_poseidon.encode_hex()
                );
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
        let hash = proof.header.inner.hash_slow();
        let expected_header_hash = format!("0x{}", hash.encode_hex());
        if expected_header_hash != proof.mmr_proof.header_hash {
            return Err(SdkError::Verification("header hash mismatch".into()));
        }

        Ok(proof.header.clone())
    }
}
