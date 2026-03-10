use std::{env, str::FromStr};

use alloy_consensus::{
    proofs::{calculate_receipt_root, calculate_transaction_root},
    transaction::TxHashRef,
    ReceiptEnvelope, TxEnvelope,
};
use alloy_eips::Encodable2718;
use alloy_primitives::B256;
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::{BlockTransactions, Transaction};
use alloy_trie::root::ordered_trie_root_with_encoder;
use anyhow::{anyhow, bail, Context, Result};
use op_alloy_consensus::{OpReceiptEnvelope, OpTxEnvelope};
use op_alloy_network::Optimism;

#[derive(Clone, Copy)]
enum Mode {
    Execution,
    OpStack,
}

impl FromStr for Mode {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "execution" => Ok(Self::Execution),
            "op-stack" => Ok(Self::OpStack),
            _ => bail!("invalid mode `{value}`, expected `execution` or `op-stack`"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 3 || args.len() > 4 {
        bail!("usage: <execution|op-stack> <rpc-url> <start-block> [count]");
    }

    let mode = Mode::from_str(&args[0])?;
    let rpc_url = &args[1];
    let start_block = args[2]
        .parse::<u64>()
        .with_context(|| format!("invalid start block `{}`", args[2]))?;
    let count = match args.get(3) {
        Some(value) => value
            .parse::<u64>()
            .with_context(|| format!("invalid block count `{value}`"))?,
        None => 1,
    };
    if count == 0 {
        bail!("block count must be greater than zero");
    }

    match mode {
        Mode::Execution => run_execution(rpc_url, start_block, count).await,
        Mode::OpStack => run_op_stack(rpc_url, start_block, count).await,
    }
}

async fn run_execution(rpc_url: &str, start_block: u64, count: u64) -> Result<()> {
    let provider = ProviderBuilder::new()
        .connect(rpc_url)
        .await
        .with_context(|| format!("failed to connect to execution RPC `{rpc_url}`"))?;
    let chain_id = provider
        .get_chain_id()
        .await
        .context("failed to fetch execution chain id")?;

    println!("mode=execution chain_id={chain_id} start_block={start_block} count={count}");

    for offset in 0..count {
        validate_execution_block(&provider, start_block + offset).await?;
    }

    println!("validated {count} execution blocks starting at {start_block}");
    Ok(())
}

async fn run_op_stack(rpc_url: &str, start_block: u64, count: u64) -> Result<()> {
    let provider = ProviderBuilder::new_with_network::<Optimism>()
        .connect(rpc_url)
        .await
        .with_context(|| format!("failed to connect to OP Stack RPC `{rpc_url}`"))?;
    let chain_id = provider
        .get_chain_id()
        .await
        .context("failed to fetch OP Stack chain id")?;

    println!("mode=op-stack chain_id={chain_id} start_block={start_block} count={count}");

    for offset in 0..count {
        validate_op_stack_block(&provider, start_block + offset).await?;
    }

    println!("validated {count} OP Stack blocks starting at {start_block}");
    Ok(())
}

async fn validate_execution_block<P: Provider>(provider: &P, block_number: u64) -> Result<()> {
    let block = provider
        .get_block_by_number(block_number.into())
        .full()
        .await
        .with_context(|| format!("failed to fetch execution block {block_number}"))?
        .ok_or_else(|| anyhow!("execution block {block_number} not found"))?;
    let txs = match block.transactions {
        BlockTransactions::Full(txs) => txs,
        _ => bail!("execution block {block_number} did not include full transactions"),
    };

    let txs = txs
        .into_iter()
        .map(Transaction::into_inner)
        .collect::<Vec<TxEnvelope>>();
    let receipts = fetch_execution_receipts(provider, block_number, &txs).await?;
    let computed_tx_root = calculate_transaction_root(&txs);
    let computed_receipt_root = calculate_receipt_root(&receipts);

    ensure_root(
        "execution",
        block_number,
        "transactions_root",
        block.header.transactions_root,
        computed_tx_root,
    )?;
    ensure_root(
        "execution",
        block_number,
        "receipts_root",
        block.header.receipts_root,
        computed_receipt_root,
    )?;

    println!(
        "[execution] block={} txs={} transactions_root=ok receipts_root=ok",
        block_number,
        txs.len()
    );
    Ok(())
}

async fn validate_op_stack_block<P: Provider<Optimism>>(
    provider: &P,
    block_number: u64,
) -> Result<()> {
    let block = provider
        .get_block_by_number(block_number.into())
        .full()
        .await
        .with_context(|| format!("failed to fetch OP Stack block {block_number}"))?
        .ok_or_else(|| anyhow!("OP Stack block {block_number} not found"))?;
    let txs = block.transactions.into_transactions_vec();
    let tx_hashes = txs
        .iter()
        .map(|tx| *tx.inner.inner.tx_hash())
        .collect::<Vec<_>>();
    let txs = txs
        .into_iter()
        .map(|tx| tx.inner.into_inner())
        .collect::<Vec<OpTxEnvelope>>();
    let receipts = fetch_op_stack_receipts(provider, block_number, &tx_hashes).await?;
    let computed_tx_root = ordered_trie_root_with_encoder(&txs, |tx, buf| tx.encode_2718(buf));
    let computed_receipt_root =
        ordered_trie_root_with_encoder(&receipts, |receipt, buf| receipt.encode_2718(buf));

    ensure_root(
        "op-stack",
        block_number,
        "transactions_root",
        block.header.transactions_root,
        computed_tx_root,
    )?;
    ensure_root(
        "op-stack",
        block_number,
        "receipts_root",
        block.header.receipts_root,
        computed_receipt_root,
    )?;

    println!(
        "[op-stack] block={} txs={} transactions_root=ok receipts_root=ok",
        block_number,
        txs.len()
    );
    Ok(())
}

async fn fetch_execution_receipts<P: Provider>(
    provider: &P,
    block_number: u64,
    txs: &[TxEnvelope],
) -> Result<Vec<ReceiptEnvelope>> {
    if let Some(receipts) = provider
        .get_block_receipts(block_number.into())
        .await
        .with_context(|| format!("failed to fetch execution block receipts for {block_number}"))?
    {
        return Ok(receipts
            .into_iter()
            .map(|receipt| receipt.map_logs(|log| log.inner).into_inner())
            .collect());
    }

    let mut receipts = Vec::with_capacity(txs.len());
    for tx in txs {
        let tx_hash = *tx.tx_hash();
        let receipt = provider
            .get_transaction_receipt(tx_hash)
            .await
            .with_context(|| format!("failed to fetch execution receipt for tx {tx_hash}"))?
            .ok_or_else(|| anyhow!("execution receipt not found for tx {tx_hash}"))?;
        receipts.push(receipt.map_logs(|log| log.inner).into_inner());
    }
    Ok(receipts)
}

async fn fetch_op_stack_receipts<P: Provider<Optimism>>(
    provider: &P,
    block_number: u64,
    tx_hashes: &[B256],
) -> Result<Vec<OpReceiptEnvelope>> {
    if let Some(receipts) = provider
        .get_block_receipts(block_number.into())
        .await
        .with_context(|| format!("failed to fetch OP Stack block receipts for {block_number}"))?
    {
        return Ok(receipts.into_iter().map(Into::into).collect());
    }

    let mut receipts = Vec::with_capacity(tx_hashes.len());
    for tx_hash in tx_hashes {
        let receipt = provider
            .get_transaction_receipt(*tx_hash)
            .await
            .with_context(|| format!("failed to fetch OP Stack receipt for tx {tx_hash}"))?
            .ok_or_else(|| anyhow!("OP Stack receipt not found for tx {tx_hash}"))?;
        receipts.push(receipt.into());
    }
    Ok(receipts)
}

fn ensure_root(
    mode: &str,
    block_number: u64,
    label: &str,
    expected: B256,
    computed: B256,
) -> Result<()> {
    if expected == computed {
        return Ok(());
    }

    bail!(
        "[{mode}] block {block_number} {label} mismatch: expected {expected}, computed {computed}"
    );
}
