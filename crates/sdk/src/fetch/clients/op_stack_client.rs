use alloy_primitives::{Address, FixedBytes, U256};
use alloy_rpc_types_eth::{EIP1186AccountProofResponse, Header as ExecutionHeader};
use bankai_types::inputs::evm::execution::{ReceiptProof, TxProof};
use mpt_generate::OpStackProofClient;

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
        _network_id: u64,
    ) -> SdkResult<TxProof> {
        let proof = OpStackProofClient::new(self.rpc_url.clone())
            .tx_proof(tx_hash)
            .await?;
        Ok(TxProof {
            network_id: proof.network_id,
            block_number: proof.block_number,
            tx_hash: proof.tx_hash,
            tx_index: proof.tx_index,
            proof: proof.proof,
            encoded_tx: proof.encoded_tx,
        })
    }

    pub async fn fetch_receipt_proof(
        &self,
        tx_hash: FixedBytes<32>,
        _network_id: u64,
    ) -> SdkResult<ReceiptProof> {
        let proof = OpStackProofClient::new(self.rpc_url.clone())
            .receipt_proof(tx_hash)
            .await?;
        Ok(ReceiptProof {
            network_id: proof.network_id,
            block_number: proof.block_number,
            tx_hash: proof.tx_hash,
            tx_index: proof.tx_index,
            proof: proof.proof,
            encoded_receipt: proof.encoded_receipt,
        })
    }

    fn execution_fetcher(&self, network_id: u64) -> ExecutionFetcher {
        ExecutionFetcher::new(self.rpc_url.clone(), network_id)
    }
}
