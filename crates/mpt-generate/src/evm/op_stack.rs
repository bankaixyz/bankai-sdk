use alloy_consensus::transaction::TxHashRef;
use alloy_provider::{Provider, ProviderBuilder};
use op_alloy_network::Optimism;

use alloy_primitives::B256;

use bankai_core::error::CoreError;

use super::proof::{
    build_receipt_proof_from_items, build_tx_proof_from_items, ReceiptProof, TxProof,
};

pub struct OpStackProofClient {
    rpc_url: String,
}

impl OpStackProofClient {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    pub async fn chain_id(&self) -> Result<u64, CoreError> {
        self.provider()
            .await?
            .get_chain_id()
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))
    }

    pub async fn tx_proof(&self, tx_hash: B256) -> Result<TxProof, CoreError> {
        let provider = self.provider().await?;
        let tx = provider
            .get_transaction_by_hash(tx_hash)
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
            .ok_or_else(|| CoreError::NotFound(format!("transaction {tx_hash} not found")))?;
        let block_number = tx
            .block_number
            .ok_or_else(|| CoreError::NotFound(format!("missing block number for tx {tx_hash}")))?;
        let tx_index = tx.transaction_index.ok_or_else(|| {
            CoreError::NotFound(format!("missing transaction index for tx {tx_hash}"))
        })?;
        let block = provider
            .get_block_by_number(block_number.into())
            .full()
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
            .ok_or_else(|| CoreError::NotFound(format!("block {block_number} not found")))?;
        let txs = block.transactions.into_transactions_vec();
        let txs = txs
            .into_iter()
            .map(|tx| tx.inner.into_inner())
            .collect::<Vec<op_alloy_consensus::OpTxEnvelope>>();

        build_tx_proof_from_items(
            self.chain_id().await?,
            block_number,
            tx_hash,
            tx_index,
            &txs,
            block.header.transactions_root,
        )
    }

    pub async fn receipt_proof(&self, tx_hash: B256) -> Result<ReceiptProof, CoreError> {
        let provider = self.provider().await?;
        let tx = provider
            .get_transaction_by_hash(tx_hash)
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
            .ok_or_else(|| CoreError::NotFound(format!("transaction {tx_hash} not found")))?;
        let block_number = tx
            .block_number
            .ok_or_else(|| CoreError::NotFound(format!("missing block number for tx {tx_hash}")))?;
        let tx_index = tx.transaction_index.ok_or_else(|| {
            CoreError::NotFound(format!("missing transaction index for tx {tx_hash}"))
        })?;
        let block = provider
            .get_block_by_number(block_number.into())
            .full()
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
            .ok_or_else(|| CoreError::NotFound(format!("block {block_number} not found")))?;
        let tx_hashes = block
            .transactions
            .as_transactions()
            .ok_or_else(|| {
                CoreError::Unsupported("block response did not include full transactions".into())
            })?
            .iter()
            .map(|tx| *tx.inner.inner.tx_hash())
            .collect::<Vec<_>>();
        let receipts = if let Some(receipts) = provider
            .get_block_receipts(block_number.into())
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
        {
            receipts
                .into_iter()
                .map(Into::into)
                .collect::<Vec<op_alloy_consensus::OpReceiptEnvelope>>()
        } else {
            let mut receipts = Vec::with_capacity(tx_hashes.len());
            for tx_hash in tx_hashes {
                let receipt = provider
                    .get_transaction_receipt(tx_hash)
                    .await
                    .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
                    .ok_or_else(|| {
                        CoreError::NotFound(format!("receipt not found for tx {tx_hash}"))
                    })?;
                receipts.push(receipt.into());
            }
            receipts
        };

        build_receipt_proof_from_items(
            self.chain_id().await?,
            block_number,
            tx_hash,
            tx_index,
            &receipts,
            block.header.receipts_root,
        )
    }

    async fn provider(&self) -> Result<impl Provider<Optimism>, CoreError> {
        ProviderBuilder::new_with_network::<Optimism>()
            .connect(self.rpc_url.as_str())
            .await
            .map_err(|e| CoreError::Provider(format!("rpc connection error: {e}")))
    }
}
