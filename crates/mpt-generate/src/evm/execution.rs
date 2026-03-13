use alloy_consensus::{transaction::TxHashRef, TxEnvelope};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::{BlockTransactions, ReceiptEnvelope, Transaction};

use alloy_primitives::B256;

use bankai_core::error::CoreError;

use super::proof::{
    build_receipt_proof_from_items, build_tx_proof_from_items, ReceiptProof, TxProof,
};

pub struct ExecutionProofClient {
    rpc_url: String,
}

impl ExecutionProofClient {
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
        let tx = self.transaction_by_hash(&provider, tx_hash).await?;
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
        let txs = match block.transactions {
            BlockTransactions::Full(txs) => txs,
            _ => {
                return Err(CoreError::Unsupported(
                    "block response did not include full transactions".into(),
                ));
            }
        };
        let txs = txs
            .into_iter()
            .map(Transaction::into_inner)
            .collect::<Vec<TxEnvelope>>();

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
        let tx = self.transaction_by_hash(&provider, tx_hash).await?;
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
        let receipts = self
            .block_receipts(&provider, block_number, &block.transactions)
            .await?;

        build_receipt_proof_from_items(
            self.chain_id().await?,
            block_number,
            tx_hash,
            tx_index,
            &receipts,
            block.header.receipts_root,
        )
    }

    async fn provider(&self) -> Result<impl Provider, CoreError> {
        ProviderBuilder::new()
            .connect(self.rpc_url.as_str())
            .await
            .map_err(|e| CoreError::Provider(format!("rpc connection error: {e}")))
    }

    async fn transaction_by_hash<P: Provider>(
        &self,
        provider: &P,
        tx_hash: B256,
    ) -> Result<Transaction, CoreError> {
        provider
            .get_transaction_by_hash(tx_hash)
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
            .ok_or_else(|| CoreError::NotFound(format!("transaction {tx_hash} not found")))
    }

    async fn block_receipts<P: Provider>(
        &self,
        provider: &P,
        block_number: u64,
        block_transactions: &BlockTransactions<Transaction>,
    ) -> Result<Vec<ReceiptEnvelope>, CoreError> {
        if let Some(receipts) = provider
            .get_block_receipts(block_number.into())
            .await
            .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
        {
            return Ok(receipts
                .into_iter()
                .map(|receipt| receipt.map_logs(|log| log.inner).into_inner())
                .collect());
        }

        let txs = block_transactions.as_transactions().ok_or_else(|| {
            CoreError::Unsupported("block response did not include full transactions".into())
        })?;

        let mut receipts = Vec::with_capacity(txs.len());
        for tx in txs {
            let tx_hash = *tx.inner.tx_hash();
            let receipt = provider
                .get_transaction_receipt(tx_hash)
                .await
                .map_err(|e| CoreError::Provider(format!("rpc error: {e}")))?
                .ok_or_else(|| {
                    CoreError::NotFound(format!("receipt not found for tx {tx_hash}"))
                })?;
            receipts.push(receipt.map_logs(|log| log.inner).into_inner());
        }
        Ok(receipts)
    }
}
