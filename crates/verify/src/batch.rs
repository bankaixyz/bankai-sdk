extern crate alloc;
use alloc::format;
use alloc::vec::Vec;

use alloy_primitives::hex::ToHexExt;

use bankai_types::proofs::HashingFunctionDto;
use bankai_types::fetch::ProofWrapper;
use bankai_types::verify::evm::EvmResults;
use bankai_types::verify::BatchResults;

use crate::bankai::stwo::verify_stwo_proof;
use crate::evm::beacon::BeaconVerifier;
use crate::evm::execution::ExecutionVerifier;
use crate::VerifyError;

pub async fn verify_batch_proof(wrapper: &ProofWrapper) -> Result<BatchResults, VerifyError> {
    let bankai_block = verify_stwo_proof(&wrapper.block_proof)?;

    let exec_root = match wrapper.hashing_function {
        HashingFunctionDto::Keccak => {
            format!("0x{}", bankai_block.execution.mmr_root_keccak.encode_hex())
        }
        HashingFunctionDto::Poseidon => format!(
            "0x{}",
            bankai_block.execution.mmr_root_poseidon.encode_hex()
        ),
    };
    let beacon_root = match wrapper.hashing_function {
        HashingFunctionDto::Keccak => {
            format!("0x{}", bankai_block.beacon.mmr_root_keccak.encode_hex())
        }
        HashingFunctionDto::Poseidon => {
            format!("0x{}", bankai_block.beacon.mmr_root_poseidon.encode_hex())
        }
    };

    let mut batch_results = BatchResults {
        evm: EvmResults {
            execution_header: Vec::new(),
            beacon_header: Vec::new(),
            account: Vec::new(),
            tx: Vec::new(),
        },
    };

    if let Some(evm) = &wrapper.evm_proofs {
        if let Some(exec_headers) = &evm.execution_header_proof {
            for proof in exec_headers {
                let result =
                    ExecutionVerifier::verify_header_proof(proof, exec_root.clone()).await?;
                batch_results.evm.execution_header.push(result);
            }
        }

        if let Some(beacon_headers) = &evm.beacon_header_proof {
            for proof in beacon_headers {
                let result =
                    BeaconVerifier::verify_header_proof(proof, beacon_root.clone()).await?;
                batch_results.evm.beacon_header.push(result);
            }
        }

        if let Some(accounts) = &evm.account_proof {
            for account in accounts {
                let result = ExecutionVerifier::verify_account_proof(
                    account,
                    &batch_results.evm.execution_header,
                )
                .await?;
                batch_results.evm.account.push(result);
            }
        }

        if let Some(tx_proofs) = &evm.tx_proof {
            for proof in tx_proofs {
                let result =
                    ExecutionVerifier::verify_tx_proof(proof, &batch_results.evm.execution_header)
                        .await?;
                batch_results.evm.tx.push(result);
            }
        }
    }

    Ok(batch_results)
}
