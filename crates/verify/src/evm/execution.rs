extern crate alloc;
use alloc::vec::Vec;

use alloy_rlp::{Decodable, Encodable};
use bankai_types::fetch::evm::execution::{AccountProof, ExecutionHeaderProof, TxProof};
use bankai_types::verify::evm::execution::{Account, ExecutionHeader, TxEnvelope};

use alloy_primitives::{keccak256, FixedBytes};
use alloy_rlp::encode as rlp_encode;
use alloy_trie::{proof::verify_proof as mpt_verify, Nibbles};

use crate::bankai::mmr::MmrVerifier;
use crate::VerifyError;

pub struct ExecutionVerifier;

impl ExecutionVerifier {
    pub fn verify_header_proof(
        proof: &ExecutionHeaderProof,
        root: FixedBytes<32>,
    ) -> Result<ExecutionHeader, VerifyError> {
        if proof.mmr_proof.root != root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        MmrVerifier::verify_mmr_proof(&proof.mmr_proof.clone())
            .map_err(|_| VerifyError::InvalidMmrProof)?;

        let hash = proof.header.inner.hash_slow();
        if hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone().inner)
    }

    pub fn verify_account_proof(
        account_proof: &AccountProof,
        headers: &[ExecutionHeader],
    ) -> Result<Account, VerifyError> {
        let header = headers
            .iter()
            .find(|h| h.number == account_proof.block_number)
            .ok_or(VerifyError::InvalidExecutionHeaderProof)?;

        if header.state_root != account_proof.state_root {
            return Err(VerifyError::InvalidStateRoot);
        }

        let expected_value = rlp_encode(account_proof.account).to_vec();
        let key = Nibbles::unpack(keccak256(account_proof.address));

        mpt_verify(
            header.state_root,
            key,
            Some(expected_value),
            account_proof.mpt_proof.iter(),
        )
        .map_err(|_| VerifyError::InvalidAccountProof)?;

        Ok(account_proof.account)
    }

    pub fn verify_tx_proof(
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
