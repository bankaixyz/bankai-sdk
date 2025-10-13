use alloy_rlp::{Decodable, Encodable};
use bankai_types::fetch::evm::execution::{AccountProof, ExecutionHeaderProof, TxProof};
use bankai_types::verify::evm::execution::{Account, ExecutionHeader, TxEnvelope};

use alloy_primitives::hex::ToHexExt;
use alloy_primitives::keccak256;
use alloy_rlp::encode as rlp_encode;
use alloy_trie::{proof::verify_proof as mpt_verify, Nibbles};

use crate::bankai::mmr::BankaiMmr;
use crate::VerifyError;

pub struct ExecutionVerifier;

impl ExecutionVerifier {
    pub async fn verify_header_proof(
        proof: &ExecutionHeaderProof,
        root: String,
    ) -> Result<ExecutionHeader, VerifyError> {
        if proof.mmr_proof.root != root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        // Verify the mmr proof
        BankaiMmr::verify_mmr_proof(proof.mmr_proof.clone())
            .await
            .map_err(|_| VerifyError::InvalidMmrProof)?;

        // Check the header hash matches the mmr proof header hash
        let hash = proof.header.inner.hash_slow();
        let expected_header_hash = format!("0x{}", hash.encode_hex());
        if expected_header_hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone().inner)
    }

    pub async fn verify_account_proof(
        account_proof: &AccountProof,
        headers: &[ExecutionHeader],
    ) -> Result<Account, VerifyError> {
        // Find the matching verified header by block number
        let header = headers
            .iter()
            .find(|h| h.number == account_proof.block_number)
            .ok_or(VerifyError::InvalidExecutionHeaderProof)?;

        // Confirm the state root matches
        if header.state_root != account_proof.state_root {
            return Err(VerifyError::InvalidStateRoot);
        }

        let expected_value = rlp_encode(account_proof.account).to_vec();

        // Compute the key: keccak(address) as nibbles
        let key = Nibbles::unpack(keccak256(account_proof.address));

        // Verify MPT proof against the state root
        mpt_verify(
            header.state_root,
            key,
            Some(expected_value),
            account_proof.mpt_proof.iter(),
        )
        .map_err(|_| VerifyError::InvalidAccountProof)?;

        Ok(account_proof.account)
    }

    pub async fn verify_tx_proof(
        proof: &TxProof,
        headers: &[ExecutionHeader],
    ) -> Result<TxEnvelope, VerifyError> {
        let header = headers
            .iter()
            .find(|h| h.number == proof.block_number)
            .ok_or(VerifyError::InvalidExecutionHeaderProof)?;

        let mut rlp_tx_index = Vec::new();
        proof.tx_index.encode(&mut rlp_tx_index);
        let key = Nibbles::unpack(&rlp_tx_index);

        mpt_verify(
            header.transactions_root,
            key,
            Some(proof.encoded_tx.clone()),
            proof.proof.iter(),
        )
        .map_err(|_| VerifyError::InvalidTxProof)?;

        let tx = TxEnvelope::decode(&mut proof.encoded_tx.as_slice())
            .map_err(|_| VerifyError::InvalidRlpDecode)?;

        Ok(tx)
    }
}
