use crate::errors::{SdkError, SdkResult};
use bankai_types::fetch::evm::execution::{AccountProof, ExecutionHeaderProof};
use bankai_types::verify::evm::execution::{Account, ExecutionHeader};

use crate::verify::bankai::mmr::BankaiMmr;
// use crate::verify::bankai::stwo::verify_stwo_proof;
use alloy_primitives::hex::ToHexExt;
use alloy_primitives::keccak256;
use alloy_rlp::encode as rlp_encode;
use alloy_trie::{proof::verify_proof as mpt_verify, Nibbles};

pub struct ExecutionVerifier;

impl ExecutionVerifier {
    pub async fn verify_header_proof(
        proof: &ExecutionHeaderProof,
        root: String,
    ) -> SdkResult<ExecutionHeader> {
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
        let hash = proof.header.inner.hash_slow();
        let expected_header_hash = format!("0x{}", hash.encode_hex());
        if expected_header_hash != proof.mmr_proof.header_hash {
            return Err(SdkError::Verification("header hash mismatch".into()));
        }

        Ok(proof.header.clone().inner)
    }

    pub async fn verify_account_proof(
        account_proof: &AccountProof,
        headers: &[ExecutionHeaderProof],
    ) -> SdkResult<Account> {
        // Find the matching verified header by block number
        let header = headers
            .iter()
            .find(|h| h.header.number == account_proof.block_number)
            .ok_or_else(|| {
                SdkError::Verification("no matching execution header for account".into())
            })?;

        // Confirm the state root matches
        if header.header.state_root != account_proof.state_root {
            return Err(SdkError::Verification("state root mismatch".into()));
        }

        let expected_value = rlp_encode(account_proof.account).to_vec();

        // Compute the key: keccak(address) as nibbles
        let key = Nibbles::unpack(keccak256(account_proof.address));

        // Verify MPT proof against the state root
        mpt_verify(
            header.header.state_root,
            key,
            Some(expected_value),
            account_proof.mpt_proof.iter(),
        )
        .map_err(|e| SdkError::Verification(format!("mpt verify error: {e}")))?;

        Ok(account_proof.account)
    }
}
