use std::time::Instant;

use crate::debug;
use crate::errors::{SdkError, SdkResult};
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::{EIP1186AccountProofResponse, Header as ExecutionHeader};
use bankai_types::inputs::evm::execution::{ReceiptProof, TxProof};
use mpt_generate::ExecutionProofClient;

pub struct ExecutionFetcher {
    pub rpc_url: String,
    pub _network_id: u64,
}

impl ExecutionFetcher {
    pub fn new(rpc_url: String, network_id: u64) -> Self {
        Self {
            rpc_url,
            _network_id: network_id,
        }
    }

    pub async fn fetch_header(&self, block_number: u64) -> SdkResult<ExecutionHeader> {
        let start = Instant::now();
        let label = format!(
            "rpc eth_getBlockByNumber endpoint={} block={}",
            debug::endpoint_label(&self.rpc_url),
            block_number
        );
        let result = async {
            let provider = ProviderBuilder::new()
                .connect(self.rpc_url.as_str())
                .await
                .map_err(|e| SdkError::Provider(format!("rpc connection error: {e}")))?;

            let block = provider
                .get_block_by_number(block_number.into())
                .await
                .map_err(|e| SdkError::Provider(format!("rpc error: {e}")))?;

            let block = block
                .ok_or_else(|| SdkError::NotFound(format!("block {block_number} not found")))?;

            Ok(block.header)
        }
        .await;
        debug::log_result(label, start, &result);
        result
    }

    pub async fn fetch_header_by_hash(
        &self,
        block_hash: FixedBytes<32>,
    ) -> SdkResult<ExecutionHeader> {
        let start = Instant::now();
        let label = format!(
            "rpc eth_getBlockByHash endpoint={} hash={}",
            debug::endpoint_label(&self.rpc_url),
            block_hash
        );
        let result = async {
            let provider = ProviderBuilder::new()
                .connect(self.rpc_url.as_str())
                .await
                .map_err(|e| SdkError::Provider(format!("rpc connection error: {e}")))?;

            let block = provider
                .get_block_by_hash(block_hash)
                .await
                .map_err(|e| SdkError::Provider(format!("rpc error: {e}")))?;

            let block =
                block.ok_or_else(|| SdkError::NotFound(format!("block {block_hash} not found")))?;

            Ok(block.header)
        }
        .await;
        debug::log_result(label, start, &result);
        result
    }

    pub async fn fetch_chain_id(&self) -> SdkResult<u64> {
        let start = Instant::now();
        let label = format!(
            "rpc eth_chainId endpoint={}",
            debug::endpoint_label(&self.rpc_url)
        );
        let result = async {
            let provider = ProviderBuilder::new()
                .connect(self.rpc_url.as_str())
                .await
                .map_err(|e| SdkError::Provider(format!("rpc connection error: {e}")))?;

            provider
                .get_chain_id()
                .await
                .map_err(|e| SdkError::Provider(format!("rpc error: {e}")))
        }
        .await;
        debug::log_result(label, start, &result);
        result
    }

    pub async fn fetch_account_proof(
        &self,
        address: Address,
        block_number: u64,
    ) -> SdkResult<EIP1186AccountProofResponse> {
        let start = Instant::now();
        let label = format!(
            "rpc eth_getProof endpoint={} address={} block={}",
            debug::endpoint_label(&self.rpc_url),
            address,
            block_number
        );
        let result = async {
            let provider = ProviderBuilder::new()
                .connect(self.rpc_url.as_str())
                .await
                .map_err(|e| SdkError::Provider(format!("rpc connection error: {e}")))?;

            let proof = provider
                .get_proof(address, vec![])
                .block_id(block_number.into())
                .await
                .map_err(|e| SdkError::Provider(format!("rpc error: {e}")))?;

            Ok(proof)
        }
        .await;
        debug::log_result(label, start, &result);
        result
    }

    /// Fetches storage slot proofs for one or more slots from the same contract.
    pub async fn fetch_storage_slot_proof(
        &self,
        address: Address,
        block_number: u64,
        slot_keys: &[U256],
    ) -> SdkResult<EIP1186AccountProofResponse> {
        let provider = ProviderBuilder::new()
            .connect(self.rpc_url.as_str())
            .await
            .map_err(|e| SdkError::Provider(format!("rpc connection error: {e}")))?;

        let keys: Vec<FixedBytes<32>> = slot_keys
            .iter()
            .map(|k| FixedBytes::from(k.to_be_bytes::<32>()))
            .collect();

        let proof = provider
            .get_proof(address, keys)
            .block_id(block_number.into())
            .await
            .map_err(|e| SdkError::Provider(format!("rpc error: {e}")))?;

        Ok(proof)
    }

    pub async fn fetch_tx_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<TxProof> {
        let proof = ExecutionProofClient::new(self.rpc_url.clone())
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

    pub async fn fetch_receipt_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<ReceiptProof> {
        let proof = ExecutionProofClient::new(self.rpc_url.clone())
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
}
