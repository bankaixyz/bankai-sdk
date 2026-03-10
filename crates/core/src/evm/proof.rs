use alloy_eips::Encodable2718;
use alloy_primitives::{Bytes, B256};
use alloy_trie::{
    proof::{verify_proof, ProofRetainer},
    root::adjust_index_for_rlp,
    HashBuilder, Nibbles,
};
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxProof {
    pub network_id: u64,
    pub block_number: u64,
    pub tx_hash: B256,
    pub tx_index: u64,
    pub proof: Vec<Bytes>,
    pub encoded_tx: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptProof {
    pub network_id: u64,
    pub block_number: u64,
    pub tx_hash: B256,
    pub tx_index: u64,
    pub proof: Vec<Bytes>,
    pub encoded_receipt: Vec<u8>,
}

pub fn build_tx_proof_from_items<T>(
    network_id: u64,
    block_number: u64,
    tx_hash: B256,
    tx_index: u64,
    txs: &[T],
    expected_root: B256,
) -> Result<TxProof, CoreError>
where
    T: Encodable2718,
{
    let (proof, encoded_tx) = build_ordered_trie_proof(txs, tx_index, expected_root, |tx, buf| {
        tx.encode_2718(buf);
    })?;

    Ok(TxProof {
        network_id,
        block_number,
        tx_hash,
        tx_index,
        proof,
        encoded_tx,
    })
}

pub fn build_receipt_proof_from_items<T>(
    network_id: u64,
    block_number: u64,
    tx_hash: B256,
    tx_index: u64,
    receipts: &[T],
    expected_root: B256,
) -> Result<ReceiptProof, CoreError>
where
    T: Encodable2718,
{
    let (proof, encoded_receipt) =
        build_ordered_trie_proof(receipts, tx_index, expected_root, |receipt, buf| {
            receipt.encode_2718(buf);
        })?;

    Ok(ReceiptProof {
        network_id,
        block_number,
        tx_hash,
        tx_index,
        proof,
        encoded_receipt,
    })
}

fn build_ordered_trie_proof<T, F>(
    items: &[T],
    target_index: u64,
    expected_root: B256,
    mut encode: F,
) -> Result<(Vec<Bytes>, Vec<u8>), CoreError>
where
    F: FnMut(&T, &mut Vec<u8>),
{
    let target_index = usize::try_from(target_index)
        .map_err(|_| CoreError::Unsupported("target index does not fit in usize".into()))?;
    if items.is_empty() {
        return Err(CoreError::InvalidMerkleTree);
    }
    if target_index >= items.len() {
        return Err(CoreError::NotFound(format!(
            "target index {target_index} out of bounds for block with {} items",
            items.len()
        )));
    }

    let target_path = ordered_trie_target_path(target_index);
    let mut hash_builder =
        HashBuilder::default().with_proof_retainer(ProofRetainer::from_iter([target_path]));
    let mut encoded_value = Vec::new();

    for sorted_index in 0..items.len() {
        let item_index = adjust_index_for_rlp(sorted_index, items.len());
        let key = ordered_trie_target_path(item_index);
        encoded_value.clear();
        encode(&items[item_index], &mut encoded_value);
        hash_builder.add_leaf(key, &encoded_value);
    }

    let root = hash_builder.root();
    if root != expected_root {
        return Err(CoreError::InvalidTrieRoot);
    }

    encoded_value.clear();
    encode(&items[target_index], &mut encoded_value);
    let proof_nodes = hash_builder.take_proof_nodes();
    let proof = proof_nodes
        .matching_nodes_sorted(&target_path)
        .into_iter()
        .map(|(_, node)| node)
        .collect::<Vec<_>>();

    verify_proof(root, target_path, Some(encoded_value.clone()), proof.iter())
        .map_err(|_| CoreError::InvalidMerkleProof)?;

    Ok((proof, encoded_value))
}

fn ordered_trie_target_path(index: usize) -> Nibbles {
    Nibbles::unpack(&alloy_rlp::encode_fixed_size(&index))
}

#[cfg(test)]
mod tests {
    use alloy_consensus::{
        proofs, Receipt, ReceiptEnvelope, ReceiptWithBloom, TxEip1559, TxEnvelope,
    };
    use alloy_eips::Encodable2718;
    use alloy_primitives::{b256, Address, Bloom, Bytes, FixedBytes, TxKind, U256};
    use alloy_trie::root::ordered_trie_root_with_encoder;
    use op_alloy_consensus::{OpReceiptEnvelope, OpTxEnvelope, OpTxType, TxDeposit};

    use super::{build_receipt_proof_from_items, build_tx_proof_from_items};

    fn sample_tx(nonce: u64) -> TxEnvelope {
        TxEnvelope::Eip1559(alloy_consensus::Signed::new_unchecked(
            TxEip1559 {
                chain_id: 1,
                nonce,
                gas_limit: 21_000,
                max_fee_per_gas: 10,
                max_priority_fee_per_gas: 1,
                to: TxKind::Call(Address::repeat_byte(0x11)),
                value: U256::from(nonce + 1),
                access_list: Default::default(),
                input: Bytes::new(),
            },
            alloy_primitives::Signature::test_signature(),
            b256!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ))
    }

    fn sample_receipt(cumulative_gas_used: u64) -> ReceiptEnvelope {
        ReceiptEnvelope::Eip1559(ReceiptWithBloom {
            receipt: Receipt {
                status: true.into(),
                cumulative_gas_used,
                logs: vec![],
            },
            logs_bloom: Bloom::ZERO,
        })
    }

    #[test]
    fn builds_transaction_proof_against_transaction_root() {
        let txs = vec![sample_tx(0), sample_tx(1), sample_tx(2)];
        let proof = build_tx_proof_from_items(
            1,
            7,
            FixedBytes::ZERO,
            1,
            &txs,
            proofs::calculate_transaction_root(&txs),
        )
        .unwrap();

        assert_eq!(proof.tx_index, 1);
        assert!(!proof.proof.is_empty());
        assert!(!proof.encoded_tx.is_empty());
    }

    #[test]
    fn builds_receipt_proof_against_receipt_root() {
        let receipts = vec![sample_receipt(21_000), sample_receipt(42_000)];
        let proof = build_receipt_proof_from_items(
            1,
            7,
            FixedBytes::ZERO,
            1,
            &receipts,
            proofs::calculate_receipt_root(&receipts),
        )
        .unwrap();

        assert_eq!(proof.tx_index, 1);
        assert!(!proof.proof.is_empty());
        assert!(!proof.encoded_receipt.is_empty());
    }

    #[test]
    fn covers_ordered_trie_boundary_indices() {
        let txs = (0..130).map(sample_tx).collect::<Vec<_>>();
        let root = proofs::calculate_transaction_root(&txs);

        for index in [0_u64, 1, 127, 128, 129] {
            let proof =
                build_tx_proof_from_items(1, 9, FixedBytes::ZERO, index, &txs, root).unwrap();
            assert_eq!(proof.tx_index, index);
            assert!(!proof.proof.is_empty());
        }
    }

    #[test]
    fn builds_op_deposit_tx_proof() {
        let tx = OpTxEnvelope::from(TxDeposit {
            source_hash: b256!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            from: Address::repeat_byte(0x33),
            to: TxKind::Call(Address::repeat_byte(0x44)),
            mint: 5,
            value: U256::from(7_u64),
            gas_limit: 50_000,
            is_system_transaction: false,
            input: Bytes::new(),
        });
        let txs = vec![tx.clone()];
        let root = ordered_trie_root_with_encoder(&txs, |item, buf| item.encode_2718(buf));
        let proof = build_tx_proof_from_items(10, 12, tx.tx_hash(), 0, &txs, root).unwrap();

        assert_eq!(proof.tx_index, 0);
        assert!(!proof.encoded_tx.is_empty());
    }

    #[test]
    fn builds_op_deposit_receipt_proof() {
        let receipt = OpReceiptEnvelope::from_parts(
            true,
            21_000,
            [].iter(),
            OpTxType::Deposit,
            Some(1),
            Some(2),
        );
        let receipts = vec![receipt.clone()];
        let root = ordered_trie_root_with_encoder(&receipts, |item, buf| item.encode_2718(buf));
        let proof =
            build_receipt_proof_from_items(10, 12, FixedBytes::ZERO, 0, &receipts, root).unwrap();

        assert_eq!(proof.tx_index, 0);
        assert!(!proof.encoded_receipt.is_empty());
    }
}
