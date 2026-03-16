extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::slice;

use alloy_primitives::U256;
use bankai_types::common::HashingFunction;
use bankai_types::inputs::evm::execution::{AccountProof, ReceiptProof, StorageSlotProof, TxProof};
use bankai_types::inputs::ProofBundle;
use bankai_types::results::evm::execution::{ReceiptEnvelope, TrieAccount, TxEnvelope};
use bankai_types::results::evm::op_stack::OpStackResults;
use bankai_types::results::evm::{
    BlockRef, EvmResults, VerifiedAccount, VerifiedReceipt, VerifiedStorageSlots,
    VerifiedTransaction,
};
use bankai_types::results::BatchResults;

use crate::bankai::stwo::verify_block_proof;
use crate::evm::beacon::BeaconVerifier;
use crate::evm::execution::ExecutionVerifier;
use crate::evm::op_stack::OpStackVerifier;
use crate::VerifyError;

/// Verifies an entire proof bundle and returns verified Ethereum and OP Stack results.
///
/// This is the main verification entry point used after `ProofBatchBuilder::execute()`.
///
/// Verification covers:
///
/// 1. the Bankai block proof
/// 2. header inclusion proofs
/// 3. account, storage, transaction, and receipt proofs that depend on those headers
///
/// # Example
///
/// ```no_run
/// use bankai_verify::verify_batch_proof;
/// use bankai_types::inputs::ProofBundle;
///
/// # fn example(proof_bundle: ProofBundle) -> Result<(), Box<dyn std::error::Error>> {
/// let results = verify_batch_proof(proof_bundle)?;
///
/// println!("Verified {} execution headers", results.evm.execution_header.len());
/// println!("Verified {} OP Stack headers", results.op_stack.header.len());
/// # Ok(())
/// # }
/// ```
pub fn verify_batch_proof(wrapper: ProofBundle) -> Result<BatchResults, VerifyError> {
    verify_block_proof(wrapper.block_proof, &wrapper.block)?;
    let bankai_block = &wrapper.block;

    let exec_root = select_root(
        wrapper.hashing_function,
        bankai_block.execution.mmr_root_keccak,
        bankai_block.execution.mmr_root_poseidon,
    );
    let beacon_root = select_root(
        wrapper.hashing_function,
        bankai_block.beacon.mmr_root_keccak,
        bankai_block.beacon.mmr_root_poseidon,
    );

    let mut batch_results = BatchResults {
        evm: EvmResults {
            execution_header: Vec::new(),
            beacon_header: Vec::new(),
            account: Vec::new(),
            tx: Vec::new(),
            receipt: Vec::new(),
            storage_slot: Vec::new(),
        },
        op_stack: OpStackResults {
            header: Vec::new(),
            account: Vec::new(),
            tx: Vec::new(),
            receipt: Vec::new(),
            storage_slot: Vec::new(),
        },
    };

    if let Some(evm) = &wrapper.evm_proofs {
        for proof in &evm.execution_header_proof {
            let result = ExecutionVerifier::verify_header_proof(proof, exec_root)?;
            batch_results.evm.execution_header.push(result);
        }

        for proof in &evm.beacon_header_proof {
            let result = BeaconVerifier::verify_header_proof(proof, beacon_root)?;
            batch_results.evm.beacon_header.push(result);
        }

        for account in &evm.account_proof {
            let result = ExecutionVerifier::verify_account_proof(
                account,
                &batch_results.evm.execution_header,
            )?;
            batch_results.evm.account.push(verified_account(account, result));
        }

        for proof in &evm.storage_slot_proof {
            let result = ExecutionVerifier::verify_storage_slot_proof(
                proof,
                &batch_results.evm.execution_header,
            )?;
            batch_results
                .evm
                .storage_slot
                .push(verified_storage_slots(proof, result));
        }

        for proof in &evm.tx_proof {
            let result =
                ExecutionVerifier::verify_tx_proof(proof, &batch_results.evm.execution_header)?;
            batch_results.evm.tx.push(verified_transaction(proof, result));
        }

        for proof in &evm.receipt_proof {
            let result = ExecutionVerifier::verify_receipt_proof(
                proof,
                &batch_results.evm.execution_header,
            )?;
            batch_results
                .evm
                .receipt
                .push(verified_receipt(proof, result));
        }
    }

    if let Some(op_stack) = &wrapper.op_stack_proofs {
        let mut verified_op_headers = BTreeMap::new();

        for proof in &op_stack.header_proof {
            let header = OpStackVerifier::verify_header_proof(
                proof,
                bankai_block.op_chains.root,
                wrapper.hashing_function,
            )?;
            verified_op_headers.insert((proof.snapshot.chain_id, header.number), header.clone());
            batch_results.op_stack.header.push(header);
        }

        for proof in &op_stack.account_proof {
            let header =
                select_op_header(&verified_op_headers, proof.network_id, proof.block_number)?;
            let result = OpStackVerifier::verify_account_proof(proof, slice::from_ref(header))?;
            batch_results
                .op_stack
                .account
                .push(verified_account(proof, result));
        }

        for proof in &op_stack.storage_slot_proof {
            let header =
                select_op_header(&verified_op_headers, proof.network_id, proof.block_number)?;
            let result =
                OpStackVerifier::verify_storage_slot_proof(proof, slice::from_ref(header))?;
            batch_results
                .op_stack
                .storage_slot
                .push(verified_storage_slots(proof, result));
        }

        for proof in &op_stack.tx_proof {
            let header =
                select_op_header(&verified_op_headers, proof.network_id, proof.block_number)?;
            let result = OpStackVerifier::verify_tx_proof(proof, slice::from_ref(header))?;
            batch_results
                .op_stack
                .tx
                .push(verified_transaction(proof, result));
        }

        for proof in &op_stack.receipt_proof {
            let header =
                select_op_header(&verified_op_headers, proof.network_id, proof.block_number)?;
            let result = OpStackVerifier::verify_receipt_proof(proof, slice::from_ref(header))?;
            batch_results
                .op_stack
                .receipt
                .push(verified_receipt(proof, result));
        }
    }

    Ok(batch_results)
}

fn block_ref(network_id: u64, block_number: u64) -> BlockRef {
    BlockRef {
        network_id,
        block_number,
    }
}

fn verified_account(proof: &AccountProof, account: TrieAccount) -> VerifiedAccount {
    VerifiedAccount {
        block: block_ref(proof.network_id, proof.block_number),
        address: proof.address,
        account,
    }
}

fn verified_storage_slots(
    proof: &StorageSlotProof,
    slots: Vec<(U256, U256)>,
) -> VerifiedStorageSlots {
    VerifiedStorageSlots {
        block: block_ref(proof.network_id, proof.block_number),
        address: proof.address,
        slots,
    }
}

fn verified_transaction(proof: &TxProof, tx: TxEnvelope) -> VerifiedTransaction {
    VerifiedTransaction {
        block: block_ref(proof.network_id, proof.block_number),
        tx_hash: proof.tx_hash,
        tx_index: proof.tx_index,
        tx,
    }
}

fn verified_receipt(proof: &ReceiptProof, receipt: ReceiptEnvelope) -> VerifiedReceipt {
    VerifiedReceipt {
        block: block_ref(proof.network_id, proof.block_number),
        tx_hash: proof.tx_hash,
        tx_index: proof.tx_index,
        receipt,
    }
}

fn select_root<T: Copy>(hashing_function: HashingFunction, keccak: T, poseidon: T) -> T {
    match hashing_function {
        HashingFunction::Keccak => keccak,
        HashingFunction::Poseidon => poseidon,
    }
}

fn select_op_header(
    headers: &BTreeMap<(u64, u64), bankai_types::results::evm::execution::ExecutionHeader>,
    network_id: u64,
    block_number: u64,
) -> Result<&bankai_types::results::evm::execution::ExecutionHeader, VerifyError> {
    headers
        .get(&(network_id, block_number))
        .ok_or(VerifyError::InvalidExecutionHeaderProof)
}

#[cfg(test)]
mod tests {
    use super::{
        select_op_header, verified_account, verified_receipt, verified_storage_slots,
        verified_transaction,
    };
    use alloy_consensus::{
        Receipt, ReceiptEnvelope, ReceiptWithBloom, Signed, TxEnvelope, TxLegacy,
    };
    use alloy_primitives::{Address, Bloom, Bytes, FixedBytes, Signature, TxKind, B256, U256};
    use bankai_types::inputs::evm::execution::{
        AccountProof, ReceiptProof, StorageSlotEntry, StorageSlotProof, TxProof,
    };
    use bankai_types::results::evm::execution::ExecutionHeader;
    use bankai_types::results::evm::execution::TrieAccount;
    use std::collections::BTreeMap;

    #[test]
    fn select_op_header_uses_network_id_and_block_number() {
        let mut headers = BTreeMap::new();
        headers.insert(
            (10, 7),
            ExecutionHeader {
                number: 7,
                beneficiary: alloy_primitives::Address::repeat_byte(0x10),
                ..Default::default()
            },
        );
        headers.insert(
            (8453, 7),
            ExecutionHeader {
                number: 7,
                beneficiary: alloy_primitives::Address::repeat_byte(0x20),
                ..Default::default()
            },
        );

        let header = select_op_header(&headers, 8453, 7).unwrap();

        assert_eq!(
            header.beneficiary,
            alloy_primitives::Address::repeat_byte(0x20)
        );
    }

    #[test]
    fn verified_account_keeps_block_and_address_identity() {
        let first = verified_account(
            &AccountProof {
                account: Default::default(),
                address: Address::repeat_byte(0x11),
                network_id: 1,
                block_number: 7,
                state_root: FixedBytes::ZERO,
                mpt_proof: vec![],
            },
            TrieAccount {
                nonce: 1,
                balance: U256::from(10u64),
                storage_root: FixedBytes::ZERO,
                code_hash: FixedBytes::ZERO,
            },
        );
        let second = verified_account(
            &AccountProof {
                account: Default::default(),
                address: Address::repeat_byte(0x11),
                network_id: 1,
                block_number: 8,
                state_root: FixedBytes::ZERO,
                mpt_proof: vec![],
            },
            TrieAccount {
                nonce: 2,
                balance: U256::from(20u64),
                storage_root: FixedBytes::ZERO,
                code_hash: FixedBytes::ZERO,
            },
        );

        assert_eq!(first.block.network_id, 1);
        assert_eq!(first.block.block_number, 7);
        assert_eq!(first.address, Address::repeat_byte(0x11));
        assert_eq!(first.account.balance, U256::from(10u64));
        assert_eq!(second.block.block_number, 8);
    }

    #[test]
    fn verified_op_stack_account_distinguishes_same_block_across_chains() {
        let first = verified_account(
            &AccountProof {
                account: Default::default(),
                address: Address::repeat_byte(0x22),
                network_id: 10,
                block_number: 99,
                state_root: FixedBytes::ZERO,
                mpt_proof: vec![],
            },
            TrieAccount {
                nonce: 1,
                balance: U256::from(5u64),
                storage_root: FixedBytes::ZERO,
                code_hash: FixedBytes::ZERO,
            },
        );
        let second = verified_account(
            &AccountProof {
                account: Default::default(),
                address: Address::repeat_byte(0x22),
                network_id: 84532,
                block_number: 99,
                state_root: FixedBytes::ZERO,
                mpt_proof: vec![],
            },
            TrieAccount {
                nonce: 1,
                balance: U256::from(6u64),
                storage_root: FixedBytes::ZERO,
                code_hash: FixedBytes::ZERO,
            },
        );

        assert_eq!(first.block.block_number, second.block.block_number);
        assert_eq!(first.block.network_id, 10);
        assert_eq!(second.block.network_id, 84532);
    }

    #[test]
    fn verified_storage_slots_keep_block_and_address_identity() {
        let result = verified_storage_slots(
            &StorageSlotProof {
                account: Default::default(),
                address: Address::repeat_byte(0x33),
                network_id: 11155111,
                block_number: 21,
                state_root: FixedBytes::ZERO,
                account_mpt_proof: vec![],
                slots: vec![StorageSlotEntry {
                    slot_key: U256::from(1u64),
                    slot_value: U256::from(2u64),
                    storage_mpt_proof: vec![],
                }],
            },
            vec![(U256::from(1u64), U256::from(2u64))],
        );

        assert_eq!(result.block.network_id, 11155111);
        assert_eq!(result.block.block_number, 21);
        assert_eq!(result.address, Address::repeat_byte(0x33));
        assert_eq!(result.slots, vec![(U256::from(1u64), U256::from(2u64))]);
    }

    #[test]
    fn verified_transaction_keeps_block_and_tx_identity() {
        let tx = TxEnvelope::Legacy(Signed::new_unchecked(
            TxLegacy {
                chain_id: Some(11155111),
                nonce: 3,
                gas_price: 10,
                gas_limit: 21_000,
                to: TxKind::Call(Address::repeat_byte(0x44)),
                value: U256::from(99u64),
                input: Bytes::new(),
            },
            Signature::new(U256::from(1u64), U256::from(2u64), false),
            B256::with_last_byte(0x55),
        ));
        let result = verified_transaction(
            &TxProof {
                network_id: 11155111,
                block_number: 34,
                tx_hash: FixedBytes::from([9u8; 32]),
                tx_index: 2,
                proof: vec![],
                encoded_tx: vec![],
            },
            tx,
        );

        assert_eq!(result.block.network_id, 11155111);
        assert_eq!(result.block.block_number, 34);
        assert_eq!(result.tx_hash, FixedBytes::from([9u8; 32]));
        assert_eq!(result.tx_index, 2);
        assert!(result.tx.is_legacy());
    }

    #[test]
    fn verified_receipt_keeps_block_and_tx_identity() {
        let receipt = ReceiptEnvelope::Eip1559(ReceiptWithBloom {
            receipt: Receipt {
                status: true.into(),
                cumulative_gas_used: 21_000,
                logs: vec![],
            },
            logs_bloom: Bloom::ZERO,
        });
        let result = verified_receipt(
            &ReceiptProof {
                network_id: 11155111,
                block_number: 55,
                tx_hash: FixedBytes::from([8u8; 32]),
                tx_index: 3,
                proof: vec![],
                encoded_receipt: vec![],
            },
            receipt,
        );

        assert_eq!(result.block.network_id, 11155111);
        assert_eq!(result.block.block_number, 55);
        assert_eq!(result.tx_hash, FixedBytes::from([8u8; 32]));
        assert_eq!(result.tx_index, 3);
        assert!(result.receipt.status());
    }
}
