use alloy_primitives::{Address, FixedBytes, U256};
use alloy_rpc_types_eth::EIP1186AccountProofResponse;
pub use alloy_rpc_types_eth::Header as ExecutionHeader;
use bankai_types::api::proofs::{HashingFunctionDto, MmrProofRequestDto};

use crate::errors::SdkResult;
use crate::fetch::{
    bankai,
    clients::{bankai_api::ApiClient, execution_client::ExecutionFetcher},
};
use alloy_rpc_types_eth::Account as AlloyAccount;
use bankai_types::fetch::evm::execution::{
    ExecutionHeaderProof, StorageSlotEntry, StorageSlotProof, TxProof,
};

/// Fetcher for EVM execution layer data with MMR proofs
///
/// This fetcher retrieves execution layer (EL) blockchain data such as headers, accounts,
/// and transactions, along with the MMR proofs needed to decommit headers from STWO proofs.
///
/// The typical flow is:
/// 1. Fetch a header with its MMR proof
/// 2. Use the MMR proof to decommit and verify the header from the STWO block proof
/// 3. Use the verified header to verify accounts/transactions via standard Merkle proofs
pub struct ExecutionChainFetcher {
    api_client: ApiClient,
    rpc_url: String,
    network_id: u64,
}

impl ExecutionChainFetcher {
    /// Creates a new execution chain fetcher
    ///
    /// # Arguments
    ///
    /// * `api_client` - The Bankai API client for fetching MMR proofs
    /// * `rpc_url` - The EVM RPC endpoint URL
    /// * `network_id` - The network ID for this chain
    pub fn new(api_client: ApiClient, rpc_url: String, network_id: u64) -> Self {
        Self {
            api_client,
            rpc_url,
            network_id,
        }
    }

    /// Fetches an execution header with its MMR proof
    ///
    /// This retrieves the execution layer header from the RPC and generates an MMR proof
    /// that can be used to decommit this header from the STWO block proof's execution MMR.
    ///
    /// # Arguments
    ///
    /// * `block_number` - The block number to fetch
    /// * `hashing_function` - The hash function to use for the MMR proof
    /// * `bankai_block_number` - The Bankai block number containing the MMR
    ///
    /// # Returns
    ///
    /// An `ExecutionHeaderProof` containing the header and MMR proof for decommitment
    pub async fn header(
        &self,
        block_number: u64,
        hashing_function: HashingFunctionDto,
        bankai_block_number: u64,
    ) -> SdkResult<ExecutionHeaderProof> {
        let header = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_header(block_number)
            .await?;
        let mmr_proof = bankai::mmr::fetch_mmr_proof(
            &self.api_client,
            &MmrProofRequestDto {
                network_id: self.network_id,
                block_number: bankai_block_number,
                hashing_function,
                header_hash: header.hash.to_string(),
            },
        )
        .await?;
        Ok(ExecutionHeaderProof {
            header,
            mmr_proof: mmr_proof.into(),
        })
    }

    /// Fetches an execution header without an MMR proof
    ///
    /// Used internally by the batch builder. For verification purposes, use `header()` instead
    /// to get the header with its MMR proof.
    pub async fn header_only(&self, block_number: u64) -> SdkResult<ExecutionHeader> {
        let header = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_header(block_number)
            .await?;
        Ok(header)
    }

    /// Returns the network ID for this fetcher
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Fetches an account proof for a specific address at a given block
    ///
    /// Returns a Merkle proof that can verify the account's state (balance, nonce, code hash,
    /// storage root) against the state root in the block header. The header itself must be
    /// verified first using an MMR proof.
    ///
    /// # Arguments
    ///
    /// * `block_number` - The block number to query
    /// * `address` - The account address
    /// * `_hashing_function` - Reserved for future use
    /// * `_bankai_block_number` - Reserved for future use
    ///
    /// # Returns
    ///
    /// An EIP-1186 account proof that can be verified against the header's state root
    pub async fn account(
        &self,
        block_number: u64,
        address: Address,
        _hashing_function: HashingFunctionDto,
        _bankai_block_number: u64,
    ) -> SdkResult<EIP1186AccountProofResponse> {
        let proof = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_account_proof(address, block_number)
            .await?;
        Ok(proof)
    }

    /// Fetches a transaction proof for a specific transaction hash
    ///
    /// Returns the transaction data along with a Merkle proof that can verify the transaction
    /// against the transactions root in the block header. The header itself must be verified
    /// first using an MMR proof.
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The transaction hash
    ///
    /// # Returns
    ///
    /// A transaction proof containing the transaction and its Merkle proof
    pub async fn tx_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<TxProof> {
        let proof = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_tx_proof(tx_hash)
            .await?;
        Ok(proof)
    }

    /// Fetches storage slot proofs for one or more slots from a contract at a given block.
    ///
    /// This calls the EIP-1186 `eth_getProof` RPC with the provided storage keys and returns
    /// the slot values along with the MPT proofs needed to verify them against the contract's
    /// storage root (which itself can be verified against the header's state root).
    ///
    /// # Arguments
    ///
    /// * `block_number` - The block number to query
    /// * `address` - The contract address
    /// * `slot_keys` - The storage keys (uint256) to query
    /// * `_hashing_function` - Reserved for future use
    /// * `_bankai_block_number` - Reserved for future use
    pub async fn storage_slot_proof(
        &self,
        block_number: u64,
        address: Address,
        slot_keys: &[U256],
        _hashing_function: HashingFunctionDto,
        _bankai_block_number: u64,
    ) -> SdkResult<StorageSlotProof> {
        let proof = ExecutionFetcher::new(self.rpc_url.clone(), self.network_id)
            .fetch_storage_slot_proof(address, block_number, slot_keys)
            .await?;

        let header = self.header_only(block_number).await?;

        let slots: Vec<StorageSlotEntry> = proof
            .storage_proof
            .into_iter()
            .map(|s| StorageSlotEntry {
                slot_key: s.key.as_b256().into(),
                slot_value: s.value,
                storage_mpt_proof: s.proof,
            })
            .collect();

        let account_state: AlloyAccount = AlloyAccount {
            balance: proof.balance,
            nonce: proof.nonce,
            code_hash: proof.code_hash,
            storage_root: proof.storage_hash,
        };

        Ok(StorageSlotProof {
            account: account_state,
            address,
            network_id: self.network_id,
            block_number,
            state_root: header.state_root,
            account_mpt_proof: proof.account_proof,
            slots,
        })
    }
}
