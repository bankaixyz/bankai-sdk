extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::slice;

use bankai_types::common::HashingFunction;
use bankai_types::inputs::ProofBundle;
use bankai_types::results::evm::op_stack::OpStackResults;
use bankai_types::results::evm::EvmResults;
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
            batch_results.evm.account.push(result);
        }

        for proof in &evm.storage_slot_proof {
            let result = ExecutionVerifier::verify_storage_slot_proof(
                proof,
                &batch_results.evm.execution_header,
            )?;
            batch_results.evm.storage_slot.push(result);
        }

        for proof in &evm.tx_proof {
            let result =
                ExecutionVerifier::verify_tx_proof(proof, &batch_results.evm.execution_header)?;
            batch_results.evm.tx.push(result);
        }

        for proof in &evm.receipt_proof {
            let result = ExecutionVerifier::verify_receipt_proof(
                proof,
                &batch_results.evm.execution_header,
            )?;
            batch_results.evm.receipt.push(result);
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
            batch_results.op_stack.account.push(result);
        }

        for proof in &op_stack.storage_slot_proof {
            let header =
                select_op_header(&verified_op_headers, proof.network_id, proof.block_number)?;
            let result =
                OpStackVerifier::verify_storage_slot_proof(proof, slice::from_ref(header))?;
            batch_results.op_stack.storage_slot.push(result);
        }

        for proof in &op_stack.tx_proof {
            let header =
                select_op_header(&verified_op_headers, proof.network_id, proof.block_number)?;
            let result = OpStackVerifier::verify_tx_proof(proof, slice::from_ref(header))?;
            batch_results.op_stack.tx.push(result);
        }

        for proof in &op_stack.receipt_proof {
            let header =
                select_op_header(&verified_op_headers, proof.network_id, proof.block_number)?;
            let result = OpStackVerifier::verify_receipt_proof(proof, slice::from_ref(header))?;
            batch_results.op_stack.receipt.push(result);
        }
    }

    Ok(batch_results)
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
    use super::select_op_header;
    use bankai_types::results::evm::execution::ExecutionHeader;
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
}
