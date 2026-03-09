use alloy_primitives::{Address, FixedBytes, U256};
use alloy_rpc_types_eth::{EIP1186AccountProofResponse, Header as ExecutionHeader};
use bankai_types::inputs::evm::execution::{ReceiptProof, TxProof};

use crate::errors::SdkResult;
use crate::fetch::clients::execution_client::ExecutionFetcher;

pub struct OpStackFetcher {
    rpc_url: String,
}

impl OpStackFetcher {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    pub async fn fetch_header(&self, block_number: u64) -> SdkResult<ExecutionHeader> {
        self.execution_fetcher(0).fetch_header(block_number).await
    }

    pub async fn fetch_header_by_hash(
        &self,
        header_hash: FixedBytes<32>,
    ) -> SdkResult<ExecutionHeader> {
        self.execution_fetcher(0)
            .fetch_header_by_hash(header_hash)
            .await
    }

    pub async fn fetch_chain_id(&self) -> SdkResult<u64> {
        self.execution_fetcher(0).fetch_chain_id().await
    }

    pub async fn fetch_account_proof(
        &self,
        address: Address,
        block_number: u64,
    ) -> SdkResult<EIP1186AccountProofResponse> {
        self.execution_fetcher(0)
            .fetch_account_proof(address, block_number)
            .await
    }

    pub async fn fetch_storage_slot_proof(
        &self,
        address: Address,
        block_number: u64,
        slot_keys: &[U256],
    ) -> SdkResult<EIP1186AccountProofResponse> {
        self.execution_fetcher(0)
            .fetch_storage_slot_proof(address, block_number, slot_keys)
            .await
    }

    pub async fn fetch_tx_proof(
        &self,
        tx_hash: FixedBytes<32>,
        network_id: u64,
    ) -> SdkResult<TxProof> {
        self.execution_fetcher(network_id)
            .fetch_tx_proof(tx_hash)
            .await
    }

    pub async fn fetch_receipt_proof(
        &self,
        tx_hash: FixedBytes<32>,
        network_id: u64,
    ) -> SdkResult<ReceiptProof> {
        self.execution_fetcher(network_id)
            .fetch_receipt_proof(tx_hash)
            .await
    }

    fn execution_fetcher(&self, network_id: u64) -> ExecutionFetcher {
        ExecutionFetcher::new(self.rpc_url.clone(), network_id)
    }
}
