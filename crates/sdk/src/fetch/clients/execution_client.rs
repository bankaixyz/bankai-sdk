use std::time::Instant;

use crate::debug;
use crate::errors::{SdkError, SdkResult};
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::{EIP1186AccountProofResponse, Header as ExecutionHeader};
use bankai_types::inputs::evm::execution::{ReceiptProof, TxProof};
use eth_trie_proofs::{tx_receipt_trie::TxReceiptsMptHandler, tx_trie::TxsMptHandler};
use url::Url;

pub struct ExecutionFetcher {
    pub rpc_url: String,
    pub network_id: u64,
}

impl ExecutionFetcher {
    pub fn new(rpc_url: String, network_id: u64) -> Self {
        Self {
            rpc_url,
            network_id,
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
            let rpc_url: Url = self
                .rpc_url
                .parse()
                .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;
            let provider = ProviderBuilder::new().connect_http(rpc_url);

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
            let rpc_url: Url = self
                .rpc_url
                .parse()
                .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;
            let provider = ProviderBuilder::new().connect_http(rpc_url);

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
            let rpc_url: Url = self
                .rpc_url
                .parse()
                .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;
            let provider = ProviderBuilder::new().connect_http(rpc_url);

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
            let rpc_url: Url = self
                .rpc_url
                .parse()
                .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;
            let provider = ProviderBuilder::new().connect_http(rpc_url);

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
        let rpc_url: Url = self
            .rpc_url
            .parse()
            .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;
        let provider = ProviderBuilder::new().connect_http(rpc_url);

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
        let rpc_url: Url = self
            .rpc_url
            .parse()
            .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;

        let mut txs_mpt_handler = TxsMptHandler::new(rpc_url).map_err(|e| {
            SdkError::Provider(format!("failed to initialize tx trie handler: {e}"))
        })?;
        txs_mpt_handler
            .build_tx_tree_from_tx_hash(tx_hash)
            .await
            .map_err(|e| SdkError::Provider(format!("failed to build tx trie: {e}")))?;

        let tx_index = txs_mpt_handler
            .tx_hash_to_tx_index(tx_hash)
            .map_err(|e| SdkError::Provider(format!("failed to locate tx index: {e}")))?;
        let proof = txs_mpt_handler
            .get_proof(tx_index)
            .map_err(|e| SdkError::Provider(format!("failed to build tx proof: {e}")))?;
        let encoded_tx = txs_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .map_err(|e| SdkError::Provider(format!("failed to verify tx proof locally: {e}")))?;

        let block_number = self.fetch_tx_block_number(tx_hash).await?;

        Ok(TxProof {
            network_id: self.network_id,
            block_number,
            tx_hash,
            tx_index,
            proof: proof.into_iter().map(|p| p.into()).collect(),
            encoded_tx,
        })
    }

    pub async fn fetch_receipt_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<ReceiptProof> {
        let rpc_url: Url = self
            .rpc_url
            .parse()
            .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;

        let mut receipts_mpt_handler = TxReceiptsMptHandler::new(rpc_url).map_err(|e| {
            SdkError::Provider(format!("failed to initialize receipt trie handler: {e}"))
        })?;
        receipts_mpt_handler
            .build_tx_receipt_tree_from_tx_hash(tx_hash)
            .await
            .map_err(|e| SdkError::Provider(format!("failed to build receipt trie: {e}")))?;

        let tx_index = receipts_mpt_handler
            .tx_hash_to_tx_index(tx_hash)
            .await
            .map_err(|e| SdkError::Provider(format!("failed to locate receipt index: {e}")))?;
        let proof = receipts_mpt_handler
            .get_proof(tx_index)
            .map_err(|e| SdkError::Provider(format!("failed to build receipt proof: {e}")))?;
        let encoded_receipt = receipts_mpt_handler
            .verify_proof(tx_index, proof.clone())
            .map_err(|e| {
                SdkError::Provider(format!("failed to verify receipt proof locally: {e}"))
            })?;

        let block_number = self.fetch_tx_block_number(tx_hash).await?;

        Ok(ReceiptProof {
            network_id: self.network_id,
            block_number,
            tx_hash,
            tx_index,
            proof: proof.into_iter().map(|node| node.into()).collect(),
            encoded_receipt,
        })
    }

    pub async fn fetch_tx_block_number(&self, tx_hash: FixedBytes<32>) -> SdkResult<u64> {
        let rpc_url: Url = self
            .rpc_url
            .parse()
            .map_err(|e| SdkError::Provider(format!("invalid rpc url: {e}")))?;
        let provider = ProviderBuilder::new().connect_http(rpc_url);

        let receipt = provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|_| SdkError::NotFound("block not found".to_string()))?
            .ok_or(SdkError::NotFound("block not found".to_string()))?;

        Ok(receipt.block_number.unwrap())
    }
}
