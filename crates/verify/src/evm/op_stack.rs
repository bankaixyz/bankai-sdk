use alloy_primitives::{FixedBytes, U256};
use bankai_core::merkle::KeccakHasher;
use bankai_types::common::HashingFunction;
use bankai_types::inputs::evm::execution::{AccountProof, ReceiptProof, StorageSlotProof, TxProof};
use bankai_types::inputs::evm::op_stack::{OpStackHeaderProof, OpStackMerkleProof};
use bankai_types::results::evm::execution::{
    Account, ExecutionHeader, ReceiptEnvelope, TxEnvelope,
};

use crate::bankai::mmr::MmrVerifier;
use crate::evm::execution::ExecutionVerifier;
use crate::VerifyError;

pub struct OpStackVerifier;

impl OpStackVerifier {
    pub fn verify_merkle_proof(
        proof: &OpStackMerkleProof,
        op_chains_root: FixedBytes<32>,
    ) -> Result<(), VerifyError> {
        let computed_root = bankai_core::merkle::hash_path::<KeccakHasher>(
            &proof.path,
            proof.leaf_hash,
            proof.merkle_leaf_index,
        );
        if computed_root != op_chains_root {
            return Err(VerifyError::InvalidMerkleProof);
        }
        Ok(())
    }

    pub fn verify_header_proof(
        proof: &OpStackHeaderProof,
        op_chains_root: FixedBytes<32>,
        hashing_function: HashingFunction,
    ) -> Result<ExecutionHeader, VerifyError> {
        let computed_leaf = proof.snapshot.commitment_leaf_hash();
        if computed_leaf != proof.merkle_proof.leaf_hash {
            return Err(VerifyError::InvalidMerkleProof);
        }

        // verify the snapshopt via merkle proof
        Self::verify_merkle_proof(&proof.merkle_proof, op_chains_root)?;

        // select the correct mmr root based on the hashing function
        let mmr_root = match hashing_function {
            HashingFunction::Keccak => proof.snapshot.mmr_root_keccak,
            HashingFunction::Poseidon => proof.snapshot.mmr_root_poseidon,
        };

        // ensure the mmr proof, uses the correct root
        if proof.mmr_proof.root != mmr_root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        // verify the mmr proof
        MmrVerifier::verify_mmr_proof(&proof.mmr_proof)?;

        // verify the header hash
        let header_hash = proof.header.hash_slow();
        if header_hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone().into())
    }

    pub fn verify_account_proof(
        proof: &AccountProof,
        headers: &[ExecutionHeader],
    ) -> Result<Account, VerifyError> {
        ExecutionVerifier::verify_account_proof(proof, headers)
    }

    pub fn verify_storage_slot_proof(
        proof: &StorageSlotProof,
        headers: &[ExecutionHeader],
    ) -> Result<Vec<(U256, U256)>, VerifyError> {
        ExecutionVerifier::verify_storage_slot_proof(proof, headers)
    }

    pub fn verify_tx_proof(
        proof: &TxProof,
        headers: &[ExecutionHeader],
    ) -> Result<TxEnvelope, VerifyError> {
        ExecutionVerifier::verify_tx_proof(proof, headers)
    }

    pub fn verify_receipt_proof(
        proof: &ReceiptProof,
        headers: &[ExecutionHeader],
    ) -> Result<ReceiptEnvelope, VerifyError> {
        ExecutionVerifier::verify_receipt_proof(proof, headers)
    }
}

#[cfg(test)]
mod tests {
    use alloy_consensus::{
        proofs::{calculate_receipt_root, calculate_transaction_root},
        Receipt, ReceiptEnvelope, ReceiptWithBloom, Signed, TxEnvelope, TxLegacy,
    };
    use alloy_primitives::Sealable;
    use alloy_primitives::{keccak256, Address, Bloom, Bytes, Signature, TxKind, B256, U256};
    use alloy_rlp::encode as rlp_encode;
    use alloy_trie::{proof::ProofRetainer, HashBuilder, Nibbles};
    use bankai_core::{
        evm::{build_receipt_proof_from_items, build_tx_proof_from_items},
        mmr,
    };
    use bankai_types::block::OpChainClient;
    use bankai_types::common::HashingFunction;
    use bankai_types::inputs::evm::execution::{
        AccountProof, ReceiptProof, StorageSlotEntry, StorageSlotProof, TxProof,
    };
    use bankai_types::inputs::evm::op_stack::{OpStackHeaderProof, OpStackMerkleProof};
    use bankai_types::inputs::evm::MmrProof;
    use bankai_types::utils::mmr::hash_to_leaf;

    use super::*;

    fn snapshot() -> OpChainClient {
        OpChainClient {
            chain_id: 10,
            block_number: 42,
            header_hash: FixedBytes::from([7u8; 32]),
            l1_submission_block: 99,
            mmr_root_keccak: FixedBytes::ZERO,
            mmr_root_poseidon: FixedBytes::ZERO,
        }
    }

    fn single_leaf_mmr_proof(
        header_hash: FixedBytes<32>,
        hashing_function: HashingFunction,
    ) -> MmrProof {
        let leaf = hash_to_leaf(header_hash, &hashing_function).0;
        let root = match hashing_function {
            HashingFunction::Keccak => {
                mmr::calculate_root_hash(&mmr::KeccakHasher::new(), 1, &[leaf]).unwrap()
            }
            HashingFunction::Poseidon => {
                mmr::calculate_root_hash(&mmr::PoseidonHasher::new(), 1, &[leaf]).unwrap()
            }
        };

        MmrProof {
            network_id: 10,
            block_number: 42,
            hashing_function,
            header_hash,
            root: FixedBytes::from(root),
            elements_index: 1,
            elements_count: 1,
            path: vec![],
            peaks: vec![FixedBytes::from(leaf)],
        }
    }

    #[test]
    fn verifies_snapshot_membership_and_mmr_proof() {
        let mut snapshot = snapshot();
        let mut consensus_header = alloy_consensus::Header::default();
        consensus_header.number = 42;
        let header: alloy_rpc_types_eth::Header<alloy_consensus::Header> =
            alloy_rpc_types_eth::Header::from_consensus(consensus_header.seal_slow(), None, None);
        let header_hash = header.hash_slow();
        let mmr_proof = single_leaf_mmr_proof(header_hash, HashingFunction::Keccak);
        snapshot.mmr_root_keccak = mmr_proof.root;
        snapshot.header_hash = header_hash;
        let leaf_hash = snapshot.commitment_leaf_hash();
        let proof = OpStackHeaderProof {
            header,
            snapshot: snapshot.clone(),
            merkle_proof: OpStackMerkleProof {
                chain_id: snapshot.chain_id,
                merkle_leaf_index: 0,
                leaf_hash,
                root: leaf_hash,
                path: vec![],
            },
            mmr_proof,
        };

        let verified =
            OpStackVerifier::verify_header_proof(&proof, leaf_hash, HashingFunction::Keccak)
                .unwrap();

        assert_eq!(verified.number, snapshot.block_number);
    }

    #[test]
    fn rejects_wrong_merkle_root() {
        let snapshot = snapshot();
        let proof = OpStackMerkleProof {
            chain_id: snapshot.chain_id,
            merkle_leaf_index: 0,
            leaf_hash: snapshot.commitment_leaf_hash(),
            root: FixedBytes::from([1u8; 32]),
            path: vec![],
        };

        assert_eq!(
            OpStackVerifier::verify_merkle_proof(&proof, FixedBytes::from([2u8; 32])),
            Err(VerifyError::InvalidMerkleProof)
        );
    }

    fn proof_nodes_to_bytes(
        proof_nodes: alloy_trie::proof::ProofNodes,
    ) -> Vec<alloy_primitives::Bytes> {
        proof_nodes
            .into_nodes_sorted()
            .into_iter()
            .map(|(_, node)| node)
            .collect()
    }

    fn test_header(number: u64) -> ExecutionHeader {
        ExecutionHeader {
            number,
            ..Default::default()
        }
    }

    fn build_state_proof(
        address: Address,
        account: alloy_rpc_types_eth::Account,
    ) -> (FixedBytes<32>, Vec<Bytes>) {
        let account_key = Nibbles::unpack(keccak256(address));
        let expected_account = rlp_encode(account).to_vec();
        let retainer = ProofRetainer::from_iter([account_key.clone()]);
        let mut hash_builder = HashBuilder::default().with_proof_retainer(retainer);
        hash_builder.add_leaf(account_key, &expected_account);
        let state_root = hash_builder.root();
        let proof = proof_nodes_to_bytes(hash_builder.take_proof_nodes());
        (state_root, proof)
    }

    #[test]
    fn verifies_storage_slot_proof_against_state_root() {
        let block_number = 7;
        let address = Address::repeat_byte(0x11);
        let slot_key = U256::ZERO;
        let slot_value = U256::from(7u64);

        let storage_key = Nibbles::unpack(keccak256(slot_key.to_be_bytes::<32>()));
        let retainer = ProofRetainer::from_iter([storage_key.clone()]);
        let mut storage_builder = HashBuilder::default().with_proof_retainer(retainer);
        storage_builder.add_leaf(storage_key, &rlp_encode(slot_value));
        let storage_root = storage_builder.root();
        let storage_mpt_proof = proof_nodes_to_bytes(storage_builder.take_proof_nodes());

        let account = alloy_rpc_types_eth::Account {
            nonce: 1,
            balance: U256::from(5u64),
            storage_root,
            code_hash: keccak256([]),
        };
        let (state_root, account_mpt_proof) = build_state_proof(address, account);
        let header = ExecutionHeader {
            number: block_number,
            state_root,
            ..Default::default()
        };
        let proof = StorageSlotProof {
            account,
            address,
            network_id: 84532,
            block_number,
            state_root,
            account_mpt_proof,
            slots: vec![StorageSlotEntry {
                slot_key,
                slot_value,
                storage_mpt_proof,
            }],
        };

        let verified = OpStackVerifier::verify_storage_slot_proof(&proof, &[header]).unwrap();

        assert_eq!(verified, vec![(slot_key, slot_value)]);
    }

    #[test]
    fn verifies_tx_proof_against_transactions_root() {
        let block_number = 9;
        let signature = Signature::new(U256::from(1u64), U256::from(2u64), false);
        let tx = TxEnvelope::Legacy(Signed::new_unchecked(
            TxLegacy {
                chain_id: Some(84532),
                nonce: 3,
                gas_price: 10,
                gas_limit: 21_000,
                to: TxKind::Call(Address::repeat_byte(0x22)),
                value: U256::from(99u64),
                input: Bytes::new(),
            },
            signature,
            B256::with_last_byte(0x44),
        ));
        let tx_root = calculate_transaction_root(&[tx.clone()]);
        let built = build_tx_proof_from_items(
            84532,
            block_number,
            *tx.tx_hash(),
            0,
            &[tx.clone()],
            tx_root,
        )
        .unwrap();
        let proof = TxProof {
            network_id: built.network_id,
            block_number: built.block_number,
            tx_hash: built.tx_hash,
            tx_index: built.tx_index,
            proof: built.proof,
            encoded_tx: built.encoded_tx,
        };
        let header = ExecutionHeader {
            number: block_number,
            transactions_root: tx_root,
            ..Default::default()
        };

        let verified = OpStackVerifier::verify_tx_proof(&proof, &[header]).unwrap();

        assert_eq!(verified.tx_type(), tx.tx_type());
        assert_eq!(verified.is_legacy(), tx.is_legacy());
    }

    #[test]
    fn verifies_receipt_proof_against_receipts_root() {
        let receipt = ReceiptEnvelope::Eip1559(ReceiptWithBloom {
            receipt: Receipt {
                status: true.into(),
                cumulative_gas_used: 21_000,
                logs: vec![],
            },
            logs_bloom: Bloom::ZERO,
        });
        let receipts_root = calculate_receipt_root(&[receipt.clone()]);
        let block_number = 11;
        let built = build_receipt_proof_from_items(
            84532,
            block_number,
            FixedBytes::ZERO,
            0,
            &[receipt.clone()],
            receipts_root,
        )
        .unwrap();
        let proof = ReceiptProof {
            network_id: built.network_id,
            block_number: built.block_number,
            tx_hash: built.tx_hash,
            tx_index: built.tx_index,
            proof: built.proof,
            encoded_receipt: built.encoded_receipt,
        };
        let header = ExecutionHeader {
            number: block_number,
            receipts_root,
            ..Default::default()
        };

        let verified = OpStackVerifier::verify_receipt_proof(&proof, &[header]).unwrap();

        assert!(verified.status());
        assert_eq!(verified.cumulative_gas_used(), 21_000);
    }

    #[test]
    fn rejects_storage_slot_proof_without_matching_header() {
        let proof = StorageSlotProof {
            account: alloy_rpc_types_eth::Account {
                storage_root: alloy_trie::EMPTY_ROOT_HASH,
                code_hash: keccak256([]),
                ..Default::default()
            },
            address: Address::ZERO,
            network_id: 84532,
            block_number: 42,
            state_root: FixedBytes::ZERO,
            account_mpt_proof: vec![],
            slots: vec![],
        };

        assert_eq!(
            OpStackVerifier::verify_storage_slot_proof(&proof, &[test_header(7)]),
            Err(VerifyError::InvalidExecutionHeaderProof)
        );
    }

    #[test]
    fn verifies_account_proof_against_state_root() {
        let address = Address::repeat_byte(0x55);
        let block_number = 13;
        let account = alloy_rpc_types_eth::Account {
            nonce: 2,
            balance: U256::from(123u64),
            storage_root: alloy_trie::EMPTY_ROOT_HASH,
            code_hash: keccak256([]),
        };
        let (state_root, mpt_proof) = build_state_proof(address, account);
        let proof = AccountProof {
            account,
            address,
            network_id: 84532,
            block_number,
            state_root,
            mpt_proof,
        };
        let header = ExecutionHeader {
            number: block_number,
            state_root,
            ..Default::default()
        };

        let verified = OpStackVerifier::verify_account_proof(&proof, &[header]).unwrap();

        assert_eq!(verified.balance, U256::from(123u64));
        assert_eq!(verified.nonce, 2);
    }
}
