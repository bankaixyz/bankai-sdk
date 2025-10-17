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
    /// use bankai_verify::evm::ExecutionVerifier;
    /// use bankai_types::fetch::evm::execution::ExecutionHeaderProof;
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

        MmrVerifier::verify_mmr_proof(&proof.mmr_proof.clone())
            .map_err(|_| VerifyError::InvalidMmrProof)?;

        let hash = proof.header.inner.hash_slow();
        if hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone().inner)
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
    /// use bankai_verify::evm::ExecutionVerifier;
    /// use bankai_types::fetch::evm::execution::AccountProof;
    /// use bankai_types::verify::evm::execution::ExecutionHeader;
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
    /// use bankai_verify::evm::ExecutionVerifier;
    /// use bankai_types::fetch::evm::execution::TxProof;
    /// use bankai_types::verify::evm::execution::ExecutionHeader;
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
