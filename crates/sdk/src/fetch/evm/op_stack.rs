use alloy_primitives::{Address, FixedBytes, U256};
use alloy_rpc_types_eth::{Account as AlloyAccount, EIP1186AccountProofResponse};
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use bankai_types::api::op_stack::{OpChainSnapshotSummaryDto, OpStackLightClientProofRequestDto};
use bankai_types::common::HashingFunction;
use bankai_types::inputs::evm::{
    execution::{ReceiptProof, StorageSlotEntry, StorageSlotProof, TxProof},
    op_stack::OpStackHeaderProof,
};

use crate::errors::{SdkError, SdkResult};
use crate::fetch::{api::ApiClient, clients::op_stack_client::OpStackFetcher};

/// Fetches OP Stack data and proof material for one configured chain.
pub struct OpStackChainFetcher {
    api_client: ApiClient,
    chain_name: String,
    op_stack_client: OpStackFetcher,
}

impl OpStackChainFetcher {
    /// Creates a fetcher for one OP Stack chain configuration.
    pub fn new(api_client: ApiClient, chain_name: String, rpc_url: String) -> Self {
        Self {
            api_client,
            chain_name,
            op_stack_client: OpStackFetcher::new(rpc_url),
        }
    }

    /// Returns the configured Bankai API chain name.
    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }

    /// Fetches an OP Stack header proof by block number.
    pub async fn header(
        &self,
        block_number: u64,
        hashing_function: HashingFunction,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpStackHeaderProof> {
        let header = self.header_only(block_number).await?;
        self.header_from_execution_header(header, hashing_function, filter)
            .await
    }

    /// Fetches an OP Stack header proof by header hash.
    pub async fn header_by_hash(
        &self,
        header_hash: FixedBytes<32>,
        hashing_function: HashingFunction,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpStackHeaderProof> {
        let header = self.header_only_by_hash(header_hash).await?;
        self.header_from_execution_header(header, hashing_function, filter)
            .await
    }

    /// Fetches the raw execution header from the configured OP RPC.
    pub async fn header_only(&self, block_number: u64) -> SdkResult<alloy_rpc_types_eth::Header> {
        self.op_stack_client.fetch_header(block_number).await
    }

    /// Fetches the raw execution header by hash from the configured OP RPC.
    pub async fn header_only_by_hash(
        &self,
        header_hash: FixedBytes<32>,
    ) -> SdkResult<alloy_rpc_types_eth::Header> {
        self.op_stack_client.fetch_header_by_hash(header_hash).await
    }

    /// Fetches an account proof from the configured OP RPC.
    pub async fn account(
        &self,
        block_number: u64,
        address: Address,
    ) -> SdkResult<EIP1186AccountProofResponse> {
        self.op_stack_client
            .fetch_account_proof(address, block_number)
            .await
    }

    /// Fetches a transaction proof from the configured OP RPC.
    pub async fn tx_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<TxProof> {
        let network_id = self.chain_id().await?;
        self.op_stack_client
            .fetch_tx_proof(tx_hash, network_id)
            .await
    }

    /// Fetches a receipt proof from the configured OP RPC.
    pub async fn receipt_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<ReceiptProof> {
        let network_id = self.chain_id().await?;
        self.op_stack_client
            .fetch_receipt_proof(tx_hash, network_id)
            .await
    }

    /// Fetches a storage proof from the configured OP RPC.
    pub async fn storage_slot_proof(
        &self,
        block_number: u64,
        address: Address,
        slot_keys: &[U256],
    ) -> SdkResult<StorageSlotProof> {
        let proof = self
            .op_stack_client
            .fetch_storage_slot_proof(address, block_number, slot_keys)
            .await?;
        let header = self.header_only(block_number).await?;
        let network_id = self.chain_id().await?;

        let slots = proof
            .storage_proof
            .into_iter()
            .map(|slot| StorageSlotEntry {
                slot_key: slot.key.as_b256().into(),
                slot_value: slot.value,
                storage_mpt_proof: slot.proof,
            })
            .collect();

        let account = AlloyAccount {
            balance: proof.balance,
            nonce: proof.nonce,
            code_hash: proof.code_hash,
            storage_root: proof.storage_hash,
        };

        Ok(StorageSlotProof {
            account,
            address,
            network_id,
            block_number,
            state_root: header.state_root,
            account_mpt_proof: proof.account_proof,
            slots,
        })
    }

    /// Returns the configured OP chain ID from the RPC.
    pub async fn chain_id(&self) -> SdkResult<u64> {
        self.op_stack_client.fetch_chain_id().await
    }

    /// Fetches the OP snapshot committed by the Bankai API for a filter.
    pub async fn snapshot(
        &self,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpChainSnapshotSummaryDto> {
        self.api_client
            .op_stack()
            .snapshot(&self.chain_name, &filter)
            .await
    }

    async fn header_from_execution_header(
        &self,
        header: alloy_rpc_types_eth::Header,
        hashing_function: HashingFunction,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpStackHeaderProof> {
        let request = OpStackLightClientProofRequestDto {
            filter,
            hashing_function,
            header_hashes: vec![header.hash.to_string()],
            proof_format: bankai_types::common::ProofFormat::Bin,
        };
        let proof = self
            .api_client
            .op_stack()
            .light_client_proof(&self.chain_name, &request)
            .await?;
        let mmr_proof = proof.mmr_proofs.into_iter().next().ok_or_else(|| {
            SdkError::NotFound(format!(
                "missing OP MMR proof for chain {}",
                self.chain_name
            ))
        })?;
        let rpc_chain_id = self.chain_id().await?;
        if proof.snapshot.chain_id != rpc_chain_id {
            return Err(SdkError::InvalidInput(format!(
                "OP chain_id mismatch for {}: rpc returned {}, proof returned {}",
                self.chain_name, rpc_chain_id, proof.snapshot.chain_id
            )));
        }

        Ok(OpStackHeaderProof {
            header,
            snapshot: proof.snapshot,
            merkle_proof: proof.merkle_proof.try_into().map_err(|e| {
                SdkError::InvalidInput(format!("invalid OP merkle proof hex from API: {e}"))
            })?,
            mmr_proof: mmr_proof.try_into().map_err(|e| {
                SdkError::InvalidInput(format!("invalid OP MMR proof hex from API: {e}"))
            })?,
        })
    }
}
