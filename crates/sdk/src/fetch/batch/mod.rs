use alloy_primitives::FixedBytes;
use alloy_primitives::{Address, U256};
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use bankai_types::api::proofs::BankaiBlockProofDto;
use bankai_types::common::{HashingFunction, ProofFormat};
use bankai_types::inputs::evm::op_stack::OpStackProofs;
use bankai_types::inputs::evm::EvmProofs;
use bankai_types::inputs::ProofBundle;
use std::time::Instant;

use crate::debug;
use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::blocks::parse_block_proof_payload;
use crate::fetch::api::ApiClient;
use crate::fetch::evm::{beacon::BeaconChainFetcher, execution::ExecutionChainFetcher};
use crate::fetch::requests::{
    AccountProofRequest, BeaconHeaderProofRequest, EvmProofsRequest, ExecutionHeaderProofRequest,
    OpStackAccountProofRequest, OpStackHeaderProofRequest, OpStackProofsRequest,
    OpStackReceiptProofRequest, OpStackStorageSlotProofRequest, OpStackTxProofRequest,
    ReceiptProofRequest, StorageSlotProofRequest, TxProofRequest,
};
use crate::Bankai;

mod ethereum;
mod op_stack;

use self::ethereum::assemble_ethereum_proofs;
use self::op_stack::assemble_op_stack_proofs;

/// Builder for the main SDK flow: collect requests, execute the batch, then verify the bundle.
pub struct ProofBatchBuilder<'a> {
    bankai: &'a Bankai,
    bankai_block_number: u64,
    hashing: HashingFunction,
    proof_format: ProofFormat,
    ethereum: EvmProofsRequest,
    op_stack: OpStackProofsRequest,
}

impl<'a> ProofBatchBuilder<'a> {
    /// Creates a new batch builder.
    pub fn new(bankai: &'a Bankai, bankai_block_number: u64, hashing: HashingFunction) -> Self {
        Self {
            bankai,
            bankai_block_number,
            hashing,
            proof_format: ProofFormat::Bin,
            ethereum: EvmProofsRequest::default(),
            op_stack: OpStackProofsRequest::default(),
        }
    }

    /// Adds an Ethereum execution header proof request for `block_number`.
    pub fn ethereum_execution_header(mut self, block_number: u64) -> Self {
        self.ethereum
            .execution_header
            .push(ExecutionHeaderProofRequest {
                network_id: self.bankai.network().execution_network_id(),
                block_number,
            });
        self
    }

    /// Adds an Ethereum beacon header proof request for `slot`.
    pub fn ethereum_beacon_header(mut self, slot: u64) -> Self {
        self.ethereum.beacon_header.push(BeaconHeaderProofRequest {
            network_id: self.bankai.network().beacon_network_id(),
            slot,
        });
        self
    }

    /// Adds an Ethereum account proof request.
    pub fn ethereum_account(mut self, block_number: u64, address: Address) -> Self {
        self.ethereum.account.push(AccountProofRequest {
            network_id: self.bankai.network().execution_network_id(),
            block_number,
            address,
        });
        self
    }

    /// Adds an Ethereum storage proof request for one or more storage slots.
    pub fn ethereum_storage_slot(
        mut self,
        block_number: u64,
        address: Address,
        slot_keys: Vec<U256>,
    ) -> Self {
        self.ethereum.storage_slot.push(StorageSlotProofRequest {
            network_id: self.bankai.network().execution_network_id(),
            block_number,
            address,
            slot_keys,
        });
        self
    }

    /// Adds an Ethereum transaction proof request by transaction hash.
    pub fn ethereum_tx(mut self, tx_hash: FixedBytes<32>) -> Self {
        self.ethereum.tx_proof.push(TxProofRequest {
            network_id: self.bankai.network().execution_network_id(),
            tx_hash,
        });
        self
    }

    /// Adds an Ethereum receipt proof request by transaction hash.
    pub fn ethereum_receipt(mut self, tx_hash: FixedBytes<32>) -> Self {
        self.ethereum.receipt_proof.push(ReceiptProofRequest {
            network_id: self.bankai.network().execution_network_id(),
            tx_hash,
        });
        self
    }

    /// Overrides the proof payload format requested from the Bankai API.
    pub fn proof_format(mut self, proof_format: ProofFormat) -> Self {
        self.proof_format = proof_format;
        self
    }

    /// Adds an OP Stack header proof request for `chain_name` and `block_number`.
    pub fn op_stack_header(mut self, chain_name: impl Into<String>, block_number: u64) -> Self {
        self.op_stack.header.push(OpStackHeaderProofRequest {
            chain_name: chain_name.into(),
            block_number: Some(block_number),
            header_hash: None,
        });
        self
    }

    /// Adds an OP Stack request for the latest committed header on `chain_name`.
    pub fn op_stack_latest_header(mut self, chain_name: impl Into<String>) -> Self {
        self.op_stack.header.push(OpStackHeaderProofRequest {
            chain_name: chain_name.into(),
            block_number: None,
            header_hash: None,
        });
        self
    }

    /// Adds an OP Stack header proof request by header hash.
    pub fn op_stack_header_by_hash(
        mut self,
        chain_name: impl Into<String>,
        header_hash: FixedBytes<32>,
    ) -> Self {
        self.op_stack.header.push(OpStackHeaderProofRequest {
            chain_name: chain_name.into(),
            block_number: None,
            header_hash: Some(header_hash),
        });
        self
    }

    /// Adds an OP Stack account proof request.
    pub fn op_stack_account(
        mut self,
        chain_name: impl Into<String>,
        block_number: u64,
        address: Address,
    ) -> Self {
        self.op_stack.account.push(OpStackAccountProofRequest {
            chain_name: chain_name.into(),
            block_number,
            address,
        });
        self
    }

    /// Adds an OP Stack storage proof request for one or more storage slots.
    pub fn op_stack_storage_slot(
        mut self,
        chain_name: impl Into<String>,
        block_number: u64,
        address: Address,
        slot_keys: Vec<U256>,
    ) -> Self {
        self.op_stack
            .storage_slot
            .push(OpStackStorageSlotProofRequest {
                chain_name: chain_name.into(),
                block_number,
                address,
                slot_keys,
            });
        self
    }

    /// Adds an OP Stack transaction proof request by transaction hash.
    pub fn op_stack_tx(mut self, chain_name: impl Into<String>, tx_hash: FixedBytes<32>) -> Self {
        self.op_stack.tx_proof.push(OpStackTxProofRequest {
            chain_name: chain_name.into(),
            tx_hash,
        });
        self
    }

    /// Adds an OP Stack receipt proof request by transaction hash.
    pub fn op_stack_receipt(
        mut self,
        chain_name: impl Into<String>,
        tx_hash: FixedBytes<32>,
    ) -> Self {
        self.op_stack
            .receipt_proof
            .push(OpStackReceiptProofRequest {
                chain_name: chain_name.into(),
                tx_hash,
            });
        self
    }

    /// Executes the batch and returns the fetched proof bundle.
    ///
    /// The returned [`ProofBundle`] must still be verified with `bankai-verify`.
    pub async fn execute(self) -> SdkResult<ProofBundle> {
        let total_start = Instant::now();
        debug::log(format!(
            "batch execute start bankai_block={} eth_requests={}/{}/{}/{}/{}/{} op_requests={}/{}/{}/{}/{}",
            self.bankai_block_number,
            self.ethereum.execution_header.len(),
            self.ethereum.beacon_header.len(),
            self.ethereum.account.len(),
            self.ethereum.storage_slot.len(),
            self.ethereum.tx_proof.len(),
            self.ethereum.receipt_proof.len(),
            self.op_stack.header.len(),
            self.op_stack.account.len(),
            self.op_stack.storage_slot.len(),
            self.op_stack.tx_proof.len(),
            self.op_stack.receipt_proof.len(),
        ));

        let api: &ApiClient = &self.bankai.api;
        let filter = BankaiBlockFilterDto::with_bankai_block_number(self.bankai_block_number);

        let ethereum_start = Instant::now();
        let ethereum = assemble_ethereum_proofs(&self, api, &filter).await?;
        debug::log(format!(
            "assembled ethereum proofs in {} ms",
            debug::elapsed_ms(ethereum_start)
        ));

        let op_stack_start = Instant::now();
        let op_stack = assemble_op_stack_proofs(&self, api, &filter).await?;
        debug::log(format!(
            "assembled op-stack proofs in {} ms",
            debug::elapsed_ms(op_stack_start)
        ));

        let block_proof_dto = match select_matching_chain_block_proof(
            self.bankai_block_number,
            &[ethereum.block_proof.as_ref(), op_stack.block_proof.as_ref()],
        )? {
            Some(value) => {
                debug::log("using shared Bankai block proof from chain proof response");
                value
            }
            None => {
                let proof_start = Instant::now();
                let proof_result = if self.proof_format == ProofFormat::Bin {
                    api.blocks().proof(self.bankai_block_number).await
                } else {
                    api.blocks()
                        .proof_with_format(self.bankai_block_number, self.proof_format)
                        .await
                };
                debug::log_result(
                    format!(
                        "api fetch shared Bankai block proof height={}",
                        self.bankai_block_number
                    ),
                    proof_start,
                    &proof_result,
                );
                proof_result?
            }
        };
        validate_bankai_block_proof(&block_proof_dto, self.bankai_block_number)?;
        let block = block_proof_dto.block.block.clone();
        let parse_start = Instant::now();
        let block_proof = parse_block_proof_payload(block_proof_dto.proof.clone())?;
        debug::log(format!(
            "parsed shared Bankai block proof in {} ms",
            debug::elapsed_ms(parse_start)
        ));

        let evm_proofs = EvmProofs {
            execution_header_proof: ethereum.execution_header_proofs,
            beacon_header_proof: ethereum.beacon_header_proofs,
            account_proof: ethereum.account_proofs,
            storage_slot_proof: ethereum.storage_slot_proofs,
            tx_proof: ethereum.tx_proofs,
            receipt_proof: ethereum.receipt_proofs,
        };
        let evm_proofs = (!evm_proofs.is_empty()).then_some(evm_proofs);

        let op_stack_proofs = OpStackProofs {
            header_proof: op_stack.header_proofs,
            account_proof: op_stack.account_proofs,
            storage_slot_proof: op_stack.storage_slot_proofs,
            tx_proof: op_stack.tx_proofs,
            receipt_proof: op_stack.receipt_proofs,
        };
        let op_stack_proofs = (!op_stack_proofs.is_empty()).then_some(op_stack_proofs);

        debug::log(format!(
            "batch execute complete in {} ms",
            debug::elapsed_ms(total_start)
        ));

        Ok(ProofBundle {
            hashing_function: self.hashing,
            block_proof,
            block,
            evm_proofs,
            op_stack_proofs,
        })
    }
}

fn select_matching_chain_block_proof(
    requested_block_number: u64,
    chain_block_proofs: &[Option<&BankaiBlockProofDto>],
) -> SdkResult<Option<BankaiBlockProofDto>> {
    let mut selected: Option<&BankaiBlockProofDto> = None;

    for proof in chain_block_proofs.iter().flatten().copied() {
        if proof.block_number != requested_block_number {
            continue;
        }
        validate_bankai_block_proof(proof, requested_block_number)?;
        if let Some(existing) = selected {
            if existing.block.block_hash != proof.block.block_hash {
                return Err(SdkError::InvalidInput(format!(
                    "conflicting Bankai block witnesses for block {}: {} != {}",
                    requested_block_number, existing.block.block_hash, proof.block.block_hash
                )));
            }
        } else {
            selected = Some(proof);
        }
    }

    Ok(selected.cloned())
}

pub(super) fn execution_fetcher<'a>(
    builder: &'a ProofBatchBuilder<'a>,
) -> SdkResult<&'a ExecutionChainFetcher> {
    builder.bankai.ethereum().execution()
}

pub(super) fn beacon_fetcher<'a>(
    builder: &'a ProofBatchBuilder<'a>,
) -> SdkResult<&'a BeaconChainFetcher> {
    builder.bankai.ethereum().beacon()
}

pub(super) fn validate_bankai_block_proof(
    block_proof: &BankaiBlockProofDto,
    expected_block_number: u64,
) -> SdkResult<()> {
    if block_proof.block_number != expected_block_number {
        return Err(SdkError::InvalidInput(format!(
            "Bankai block proof block_number mismatch: expected {}, got {}",
            expected_block_number, block_proof.block_number
        )));
    }
    if block_proof.block.block.block_number != expected_block_number {
        return Err(SdkError::InvalidInput(format!(
            "Bankai block witness block_number mismatch: expected {}, got {}",
            expected_block_number, block_proof.block.block.block_number
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, FixedBytes, U256};
    use bankai_types::api::proofs::{BankaiBlockProofDto, BlockProofPayloadDto};
    use bankai_types::block::{BankaiBlock, BankaiBlockOutput};

    use super::{select_matching_chain_block_proof, ProofBatchBuilder};
    use crate::{Bankai, HashingFunction, Network};

    fn block_output(block_number: u64, hash_byte: u8) -> BankaiBlockOutput {
        BankaiBlockOutput {
            block_hash: FixedBytes::from([hash_byte; 32]),
            block: BankaiBlock {
                block_number,
                ..Default::default()
            },
        }
    }

    #[test]
    fn op_stack_builder_collects_requests() {
        let hash = FixedBytes::from([7u8; 32]);
        let sdk = Bankai::new(Network::Local, None, None, None);
        let builder = ProofBatchBuilder::new(&sdk, 7, HashingFunction::Keccak)
            .op_stack_header("base", 12)
            .op_stack_latest_header("base")
            .op_stack_header_by_hash("base", hash)
            .op_stack_account("base", 12, Address::ZERO)
            .op_stack_storage_slot("base", 12, Address::ZERO, vec![U256::from(1u64)])
            .op_stack_tx("base", hash)
            .op_stack_receipt("base", hash);

        assert_eq!(builder.op_stack.header.len(), 3);
        assert_eq!(builder.op_stack.header[0].block_number, Some(12));
        assert!(builder.op_stack.header[0].header_hash.is_none());
        assert_eq!(builder.op_stack.account.len(), 1);
        assert_eq!(builder.op_stack.storage_slot.len(), 1);
        assert_eq!(builder.op_stack.tx_proof.len(), 1);
        assert_eq!(builder.op_stack.receipt_proof.len(), 1);
    }

    #[test]
    fn select_matching_chain_block_proof_prefers_matching_block_number() {
        let requested_block_number = 80;
        let mismatched = BankaiBlockProofDto {
            block_number: 79,
            block: block_output(79, 1),
            proof: BlockProofPayloadDto::Json(serde_json::json!({ "proof": "mismatch" })),
        };
        let matching = BankaiBlockProofDto {
            block_number: requested_block_number,
            block: block_output(requested_block_number, 2),
            proof: BlockProofPayloadDto::Json(serde_json::json!({ "proof": "match" })),
        };

        let proof = select_matching_chain_block_proof(
            requested_block_number,
            &[Some(&mismatched), Some(&matching)],
        )
        .unwrap();

        match proof {
            Some(BankaiBlockProofDto {
                proof: BlockProofPayloadDto::Json(value),
                ..
            }) => {
                assert_eq!(value, serde_json::json!({ "proof": "match" }));
            }
            other => panic!("expected matching json proof, got {other:?}"),
        }
    }

    #[test]
    fn select_matching_chain_block_proof_ignores_mismatched_nested_proofs() {
        let mismatched = BankaiBlockProofDto {
            block_number: 79,
            block: block_output(79, 1),
            proof: BlockProofPayloadDto::Json(serde_json::json!({ "proof": "mismatch" })),
        };

        let proof = select_matching_chain_block_proof(80, &[Some(&mismatched), None]).unwrap();

        assert!(proof.is_none());
    }
}
