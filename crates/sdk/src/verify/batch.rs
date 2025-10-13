use alloy_primitives::hex::ToHexExt;

use bankai_types::api::proofs::HashingFunctionDto;
use bankai_types::fetch::evm::execution::ExecutionHeaderProof;
use bankai_types::fetch::ProofWrapper;
// types are referenced via `batch_results`; direct imports unnecessary
use bankai_types::verify::evm::EvmResults;
use bankai_types::verify::BatchResults;

use crate::errors::{SdkError, SdkResult};
use crate::verify::bankai::stwo::verify_stwo_proof;
use crate::verify::evm::beacon::BeaconVerifier;
use crate::verify::evm::execution::ExecutionVerifier;

pub async fn verify_wrapper(wrapper: &ProofWrapper) -> SdkResult<BatchResults> {
    // Verify the block proof and get the Bankai block commitments
    let bankai_block = verify_stwo_proof(&wrapper.block_proof)
        .map_err(|e| SdkError::Verification(format!("stwo verification failed: {e}")))?;

    let exec_root = match wrapper.hashing_function {
        HashingFunctionDto::Keccak => format!("0x{}", bankai_block.execution.mmr_root_keccak.encode_hex()),
        HashingFunctionDto::Poseidon => format!("0x{}", bankai_block.execution.mmr_root_poseidon.encode_hex()),
    };
    let beacon_root = match wrapper.hashing_function {
        HashingFunctionDto::Keccak => format!("0x{}", bankai_block.beacon.mmr_root_keccak.encode_hex()),
        HashingFunctionDto::Poseidon => format!("0x{}", bankai_block.beacon.mmr_root_poseidon.encode_hex()),
    };

    let mut batch_results = BatchResults {
        evm: EvmResults {
            execution_header: Vec::new(),
            beacon_header: Vec::new(),
            account: Vec::new(),
        },
    };

    if let Some(evm) = &wrapper.evm_proofs {
        // Verify execution headers
        if let Some(exec_headers) = &evm.execution_header_proof {
            for proof in exec_headers {
                let result = ExecutionVerifier::verify_header_proof(proof, exec_root.clone()).await?;
                batch_results.evm.execution_header.push(result);
            }
        }

        // Verify beacon headers
        if let Some(beacon_headers) = &evm.beacon_header_proof {
            for proof in beacon_headers {   
                let result = BeaconVerifier::verify_header_proof(proof, beacon_root.clone()).await?;
                batch_results.evm.beacon_header.push(result);
            }
        }

        // Verify account proofs (requires verified execution headers)
        if let Some(accounts) = &evm.account_proof {
            let exec_headers_slice: &[ExecutionHeaderProof] = if let Some(ref v) = evm.execution_header_proof {
                v.as_slice()
            } else {
                &[]
            };
            for account in accounts {
                let result = ExecutionVerifier::verify_account_proof(account, exec_headers_slice).await?;
                batch_results.evm.account.push(result);
            }
        }
    }

    Ok(batch_results)
}


