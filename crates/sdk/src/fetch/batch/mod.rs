use std::collections::BTreeMap;

use alloy_primitives::FixedBytes;
use alloy_primitives::{Address, U256};
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use bankai_types::api::op_stack::OpChainSnapshotSummaryDto;
use bankai_types::common::{HashingFunction, ProofFormat};
use bankai_types::inputs::evm::op_stack::OpStackProofs;
use bankai_types::inputs::evm::EvmProofs;
use bankai_types::inputs::ProofBundle;

use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::blocks::parse_block_proof_payload;
use crate::fetch::api::ApiClient;
use crate::fetch::evm::{
    beacon::BeaconChainFetcher, execution::ExecutionChainFetcher, op_stack::OpStackChainFetcher,
};
use crate::fetch::requests::{
    AccountProofRequest, BeaconHeaderProofRequest, EvmProofsRequest, ExecutionHeaderProofRequest,
    OpStackAccountProofRequest, OpStackHeaderProofRequest, OpStackProofsRequest,
    OpStackReceiptProofRequest, OpStackStorageSlotProofRequest, OpStackTxProofRequest,
    ReceiptProofRequest, StorageSlotProofRequest, TxProofRequest,
};
use crate::{Bankai, Network};

mod ethereum;
mod op_stack;

use self::ethereum::assemble_ethereum_proofs;
use self::op_stack::assemble_op_stack_proofs;

/// Builder for the main SDK flow: collect requests, execute the batch, then verify the bundle.
pub struct ProofBatchBuilder<'a> {
    bankai: &'a Bankai,
    network: Network,
    bankai_block_number: u64,
    hashing: HashingFunction,
    proof_format: ProofFormat,
    ethereum: EvmProofsRequest,
    op_stack: OpStackProofsRequest,
}

impl<'a> ProofBatchBuilder<'a> {
    /// Creates a new batch builder.
    pub fn new(
        bankai: &'a Bankai,
        network: Network,
        bankai_block_number: u64,
        hashing: HashingFunction,
    ) -> Self {
        Self {
            bankai,
            network,
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
                network_id: self.network.execution_network_id(),
                block_number,
            });
        self
    }

    /// Adds an Ethereum beacon header proof request for `slot`.
    pub fn ethereum_beacon_header(mut self, slot: u64) -> Self {
        self.ethereum.beacon_header.push(BeaconHeaderProofRequest {
            network_id: self.network.beacon_network_id(),
            slot,
        });
        self
    }

    /// Adds an Ethereum account proof request.
    pub fn ethereum_account(mut self, block_number: u64, address: Address) -> Self {
        self.ethereum.account.push(AccountProofRequest {
            network_id: self.network.execution_network_id(),
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
            network_id: self.network.execution_network_id(),
            block_number,
            address,
            slot_keys,
        });
        self
    }

    /// Adds an Ethereum transaction proof request by transaction hash.
    pub fn ethereum_tx(mut self, tx_hash: FixedBytes<32>) -> Self {
        self.ethereum.tx_proof.push(TxProofRequest {
            network_id: self.network.execution_network_id(),
            tx_hash,
        });
        self
    }

    /// Adds an Ethereum receipt proof request by transaction hash.
    pub fn ethereum_receipt(mut self, tx_hash: FixedBytes<32>) -> Self {
        self.ethereum.receipt_proof.push(ReceiptProofRequest {
            network_id: self.network.execution_network_id(),
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
        let api: &ApiClient = &self.bankai.api;
        let full_block = api.blocks().full(self.bankai_block_number).await?;
        let block = full_block.block.to_block();
        let filter = BankaiBlockFilterDto::with_bankai_block_number(self.bankai_block_number);

        let ethereum = assemble_ethereum_proofs(&self, api, &filter).await?;
        let op_stack = assemble_op_stack_proofs(&self, api, &filter).await?;

        let block_proof_value = match ethereum.block_proof_value.or(op_stack.block_proof_value) {
            Some(value) => value,
            None => {
                if self.proof_format == ProofFormat::Bin {
                    api.blocks().proof(self.bankai_block_number).await?.proof
                } else {
                    api.blocks()
                        .proof_with_format(self.bankai_block_number, self.proof_format)
                        .await?
                        .proof
                }
            }
        };
        let block_proof = parse_block_proof_payload(block_proof_value)?;

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

        Ok(ProofBundle {
            hashing_function: self.hashing,
            block_proof,
            block,
            evm_proofs,
            op_stack_proofs,
        })
    }
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

pub(super) async fn get_or_fetch_op_snapshot(
    snapshots: &mut BTreeMap<String, OpChainSnapshotSummaryDto>,
    fetcher: &OpStackChainFetcher,
    filter: BankaiBlockFilterDto,
) -> SdkResult<OpChainSnapshotSummaryDto> {
    if let Some(snapshot) = snapshots.get(fetcher.chain_name()) {
        return Ok(snapshot.clone());
    }

    let snapshot = fetcher.snapshot(filter).await?;
    let rpc_chain_id = fetcher.chain_id().await?;
    if rpc_chain_id != snapshot.chain_id {
        return Err(SdkError::InvalidInput(format!(
            "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
            fetcher.chain_name(),
            rpc_chain_id,
            snapshot.chain_id
        )));
    }
    snapshots.insert(fetcher.chain_name().to_string(), snapshot.clone());
    Ok(snapshot)
}

pub(super) fn op_snapshot_to_witness(
    snapshot: OpChainSnapshotSummaryDto,
) -> SdkResult<bankai_types::block::OpChainClient> {
    Ok(bankai_types::block::OpChainClient {
        chain_id: snapshot.chain_id,
        block_number: snapshot.end_height,
        header_hash: parse_fixed_bytes(&snapshot.header_hash)?,
        l1_submission_block: snapshot.l1_submission_block,
        mmr_root_keccak: parse_fixed_bytes(&snapshot.mmr_roots.keccak_root)?,
        mmr_root_poseidon: parse_fixed_bytes(&snapshot.mmr_roots.poseidon_root)?,
    })
}

pub(super) fn parse_fixed_bytes(value: &str) -> SdkResult<FixedBytes<32>> {
    value
        .parse()
        .map_err(|e| SdkError::InvalidInput(format!("invalid fixed bytes value {value}: {e}")))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, FixedBytes, U256};

    use super::ProofBatchBuilder;
    use crate::{Bankai, HashingFunction, Network};

    #[test]
    fn op_stack_builder_collects_requests() {
        let hash = FixedBytes::from([7u8; 32]);
        let sdk = Bankai::new(Network::Local, None, None, None);
        let builder = ProofBatchBuilder::new(&sdk, Network::Local, 7, HashingFunction::Keccak)
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
}
