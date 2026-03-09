extern crate alloc;
use alloc::vec::Vec;

use alloy_rlp::{Decodable, Encodable};
use bankai_types::inputs::evm::execution::{
    AccountProof, ExecutionHeaderProof, ReceiptProof, TxProof,
};
use bankai_types::results::evm::execution::{
    Account, ExecutionHeader, ReceiptEnvelope, TxEnvelope,
};

use alloy_primitives::{keccak256, FixedBytes};
use alloy_rlp::encode as rlp_encode;
use alloy_trie::{proof::verify_proof as mpt_verify, Nibbles};

use crate::bankai::mmr::MmrVerifier;
use crate::VerifyError;

/// Verifier for EVM execution layer proofs
///
/// Provides methods to verify execution headers, account states, and transactions
/// against trusted MMR roots and header state roots. All verifications are cryptographically
/// sound and establish trust through the STWO proof → MMR proof → Merkle proof chain.
pub struct ExecutionVerifier;

impl ExecutionVerifier {
    /// Verifies an execution layer header using an MMR inclusion proof
    ///
    /// This method establishes trust in an execution header by:
    /// 1. Verifying the MMR root matches the expected root from the STWO proof
    /// 2. Verifying the MMR inclusion proof
    /// 3. Verifying the header hash matches the value committed in the MMR
    ///
    /// Once verified, the header can be trusted and used to verify accounts and transactions.
    ///
    /// # Arguments
    ///
    /// * `proof` - The execution header proof containing the header and MMR inclusion proof
    /// * `root` - The trusted MMR root from the verified STWO proof
    ///
    /// # Returns
    ///
    /// Returns the verified `ExecutionHeader` containing all block data (number, timestamp,
    /// state root, transactions root, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `InvalidMmrRoot`: The MMR root in the proof doesn't match the expected root
    /// - `InvalidMmrProof`: The MMR inclusion proof is invalid
    /// - `InvalidHeaderHash`: The header hash doesn't match the MMR commitment
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bankai_verify::evm::execution::ExecutionVerifier;
    /// use bankai_types::inputs::evm::execution::ExecutionHeaderProof;
    /// use alloy_primitives::FixedBytes;
    ///
    /// # fn example(proof: ExecutionHeaderProof, mmr_root: FixedBytes<32>) -> Result<(), Box<dyn std::error::Error>> {
    /// let verified_header = ExecutionVerifier::verify_header_proof(&proof, mmr_root)?;
    /// println!("Verified block {}", verified_header.number);
    /// println!("State root: {:?}", verified_header.state_root);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_header_proof(
        proof: &ExecutionHeaderProof,
        root: FixedBytes<32>,
    ) -> Result<ExecutionHeader, VerifyError> {
        if proof.mmr_proof.root != root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        MmrVerifier::verify_mmr_proof(&proof.mmr_proof)?;

        let hash = proof.header.hash_slow();
        if hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone().into())
    }

    /// Verifies an account's state using a Merkle Patricia Trie proof
    ///
    /// This method verifies an account's state (balance, nonce, code hash, storage root)
    /// against a previously verified execution header. The verification uses a Merkle Patricia
    /// Trie proof to establish that the account state is included in the header's state root.
    ///
    /// # Arguments
    ///
    /// * `account_proof` - The account proof containing the account state and MPT proof
    /// * `headers` - List of previously verified execution headers. Must contain the header
    ///   for the block number referenced in the account proof
    ///
    /// # Returns
    ///
    /// Returns the verified `Account` containing:
    /// - Balance (in wei)
    /// - Nonce (transaction count)
    /// - Code hash (contract code hash, or empty for EOAs)
    /// - Storage root (Merkle root of contract storage)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `InvalidExecutionHeaderProof`: The referenced header is not in the verified headers list
    /// - `InvalidStateRoot`: The state root in the proof doesn't match the header's state root
    /// - `InvalidAccountProof`: The MPT proof verification failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bankai_verify::evm::execution::ExecutionVerifier;
    /// use bankai_types::inputs::evm::execution::AccountProof;
    /// use bankai_types::results::evm::execution::ExecutionHeader;
    ///
    /// # fn example(
    /// #     account_proof: AccountProof,
    /// #     verified_headers: Vec<ExecutionHeader>
    /// # ) -> Result<(), Box<dyn std::error::Error>> {
    /// let account = ExecutionVerifier::verify_account_proof(&account_proof, &verified_headers)?;
    /// println!("Account balance: {} wei", account.balance);
    /// println!("Account nonce: {}", account.nonce);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_account_proof(
        account_proof: &AccountProof,
        headers: &[ExecutionHeader],
    ) -> Result<Account, VerifyError> {
        let header = Self::header_for_block(headers, account_proof.block_number)?;

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

    /// Verifies one or more storage slots from the same contract using Merkle Patricia Trie proofs.
    ///
    /// This method establishes that storage slot values are committed in the state of a given
    /// block by:
    /// 1. Verifying the contract account is included in the block's state trie (against the
    ///    verified header's `state_root`)
    /// 2. Verifying each storage slot is included in the contract's storage trie (against the
    ///    account's `storage_root`)
    ///
    /// # Arguments
    ///
    /// * `slot_proof` - The storage slot proof containing the account proof and individual
    ///   storage slot proofs
    /// * `headers` - List of previously verified execution headers. Must contain the header
    ///   for the block number referenced in the storage slot proof
    ///
    /// # Returns
    ///
    /// Returns a vector of verified (slot_key, slot_value) pairs in the same order as the input.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `InvalidExecutionHeaderProof`: The referenced header is not in the verified headers list
    /// - `InvalidStateRoot`: The state root in the proof doesn't match the header's state root
    /// - `InvalidAccountProof`: The account MPT proof verification failed
    /// - `InvalidStorageProof`: Any storage slot MPT proof verification failed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bankai_verify::evm::execution::ExecutionVerifier;
    /// use bankai_types::inputs::evm::execution::StorageSlotProof;
    /// use bankai_types::results::evm::execution::ExecutionHeader;
    ///
    /// # fn example(
    /// #     slot_proof: StorageSlotProof,
    /// #     verified_headers: Vec<ExecutionHeader>
    /// # ) -> Result<(), Box<dyn std::error::Error>> {
    /// let values = ExecutionVerifier::verify_storage_slot_proof(&slot_proof, &verified_headers)?;
    /// for (key, value) in values {
    ///     println!("Slot {:?} = {}", key, value);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_storage_slot_proof(
        slot_proof: &bankai_types::inputs::evm::execution::StorageSlotProof,
        headers: &[ExecutionHeader],
    ) -> Result<Vec<(alloy_primitives::U256, alloy_primitives::U256)>, VerifyError> {
        let header = Self::header_for_block(headers, slot_proof.block_number)?;

        if header.state_root != slot_proof.state_root {
            return Err(VerifyError::InvalidStateRoot);
        }

        let expected_account = rlp_encode(slot_proof.account).to_vec();
        let account_key = Nibbles::unpack(keccak256(slot_proof.address));
        mpt_verify(
            header.state_root,
            account_key,
            Some(expected_account),
            slot_proof.account_mpt_proof.iter(),
        )
        .map_err(|_| VerifyError::InvalidAccountProof)?;

        let mut results = Vec::with_capacity(slot_proof.slots.len());
        for slot in &slot_proof.slots {
            Self::verify_storage_slot_entry(
                slot_proof.account.storage_root,
                slot.slot_key,
                slot.slot_value,
                &slot.storage_mpt_proof,
            )?;
            results.push((slot.slot_key, slot.slot_value));
        }

        Ok(results)
    }

    /// Internal helper to verify a single storage slot entry against a storage root
    fn verify_storage_slot_entry(
        storage_root: FixedBytes<32>,
        slot_key: alloy_primitives::U256,
        slot_value: alloy_primitives::U256,
        storage_mpt_proof: &[alloy_primitives::Bytes],
    ) -> Result<(), VerifyError> {
        let slot_key_bytes = slot_key.to_be_bytes::<32>();
        let storage_key = Nibbles::unpack(keccak256(slot_key_bytes));
        let expected_storage_value = if slot_value.is_zero() {
            None
        } else {
            Some(rlp_encode(slot_value).to_vec())
        };

        mpt_verify(
            storage_root,
            storage_key,
            expected_storage_value,
            storage_mpt_proof.iter(),
        )
        .map_err(|_| VerifyError::InvalidStorageProof)?;

        Ok(())
    }

    /// Verifies a transaction using a Merkle Patricia Trie proof
    ///
    /// This method verifies that a transaction was included in a specific block by validating
    /// an MPT proof against a previously verified execution header. The proof establishes that
    /// the transaction exists at a specific index in the block's transaction list.
    ///
    /// # Arguments
    ///
    /// * `proof` - The transaction proof containing the encoded transaction and MPT proof
    /// * `headers` - List of previously verified execution headers. Must contain the header
    ///   for the block number referenced in the transaction proof
    ///
    /// # Returns
    ///
    /// Returns the verified `TxEnvelope` containing the full transaction data including:
    /// - Transaction type (Legacy, EIP-1559, EIP-2930, etc.)
    /// - From/to addresses
    /// - Value transferred
    /// - Gas limit and gas price
    /// - Input data
    /// - Signature (v, r, s)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `InvalidExecutionHeaderProof`: The referenced header is not in the verified headers list
    /// - `InvalidTxProof`: The MPT proof verification failed
    /// - `InvalidRlpDecode`: The transaction data could not be decoded
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bankai_verify::evm::execution::ExecutionVerifier;
    /// use bankai_types::inputs::evm::execution::TxProof;
    /// use bankai_types::results::evm::execution::ExecutionHeader;
    ///
    /// # fn example(
    /// #     tx_proof: TxProof,
    /// #     verified_headers: Vec<ExecutionHeader>
    /// # ) -> Result<(), Box<dyn std::error::Error>> {
    /// let tx = ExecutionVerifier::verify_tx_proof(&tx_proof, &verified_headers)?;
    /// println!("Verified transaction in block {}", tx_proof.block_number);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_tx_proof(
        proof: &TxProof,
        headers: &[ExecutionHeader],
    ) -> Result<TxEnvelope, VerifyError> {
        let header = Self::header_for_block(headers, proof.block_number)?;

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

    pub fn verify_receipt_proof(
        proof: &ReceiptProof,
        headers: &[ExecutionHeader],
    ) -> Result<ReceiptEnvelope, VerifyError> {
        let header = Self::header_for_block(headers, proof.block_number)?;

        let mut rlp_tx_index = Vec::new();
        proof.tx_index.encode(&mut rlp_tx_index);
        let key = Nibbles::unpack(&rlp_tx_index);

        mpt_verify(
            header.receipts_root,
            key,
            Some(proof.encoded_receipt.clone()),
            proof.proof.iter(),
        )
        .map_err(|_| VerifyError::InvalidReceiptProof)?;

        let receipt = ReceiptEnvelope::decode(&mut proof.encoded_receipt.as_slice())
            .map_err(|_| VerifyError::InvalidRlpDecode)?;

        Ok(receipt)
    }

    fn header_for_block<'a>(
        headers: &'a [ExecutionHeader],
        block_number: u64,
    ) -> Result<&'a ExecutionHeader, VerifyError> {
        headers
            .iter()
            .find(|h| h.number == block_number)
            .ok_or(VerifyError::InvalidExecutionHeaderProof)
    }
}

#[cfg(test)]
mod tests {
    use alloy_consensus::{
        proofs::calculate_receipt_root, Receipt, ReceiptEnvelope, ReceiptWithBloom,
    };
    use alloy_primitives::{Bloom, FixedBytes};
    use eth_trie_proofs::{tx_receipt::ConsensusTxReceipt, tx_receipt_trie::TxReceiptsMptHandler};
    use url::Url;

    use super::*;

    #[test]
    fn verifies_receipt_proof_against_receipts_root() {
        let receipt = ReceiptEnvelope::Eip1559(ReceiptWithBloom {
            receipt: Receipt {
                status: true.into(),
                cumulative_gas_used: 21_000,
                logs: vec![],
            },
            logs_bloom: Bloom::ZERO,
        });
        let receipts_root = calculate_receipt_root(&[receipt.clone()]);

        let mut trie =
            TxReceiptsMptHandler::new(Url::parse("http://localhost:8545").unwrap()).unwrap();
        trie.build_trie(vec![ConsensusTxReceipt(receipt.clone())], receipts_root)
            .unwrap();

        let proof_nodes = trie.get_proof(0).unwrap();
        let encoded_receipt = trie.verify_proof(0, proof_nodes.clone()).unwrap();

        let proof = ReceiptProof {
            network_id: 1,
            block_number: 7,
            tx_hash: FixedBytes::ZERO,
            tx_index: 0,
            proof: proof_nodes.into_iter().map(Into::into).collect(),
            encoded_receipt: encoded_receipt.clone(),
        };
        let header = ExecutionHeader {
            number: 7,
            receipts_root,
            ..Default::default()
        };

        let verified = ExecutionVerifier::verify_receipt_proof(&proof, &[header]).unwrap();
        match verified {
            ReceiptEnvelope::Eip1559(receipt) => {
                assert_eq!(receipt.receipt.cumulative_gas_used, 21_000);
                assert_eq!(receipt.receipt.status, true.into());
                assert!(receipt.receipt.logs.is_empty());
            }
            other => panic!("expected EIP-1559 receipt, got {other:?}"),
        }
    }
}
