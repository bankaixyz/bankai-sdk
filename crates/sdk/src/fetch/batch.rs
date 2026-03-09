use std::collections::{BTreeMap, BTreeSet};

use alloy_primitives::FixedBytes;
use alloy_primitives::{hex::ToHexExt, Address, U256};
use alloy_rpc_types_beacon::header::HeaderResponse;
use alloy_rpc_types_eth::{Account as AlloyAccount, Header as ExecutionHeader};
use bankai_types::api::ethereum::{BankaiBlockFilterDto, EthereumLightClientProofRequestDto};
use bankai_types::api::op_stack::OpStackLightClientProofRequestDto;
use bankai_types::api::proofs::BlockProofPayloadDto;
use bankai_types::common::{HashingFunction, ProofFormat};
use bankai_types::inputs::evm::op_stack::{OpStackHeaderProof, OpStackProofs};
use bankai_types::inputs::evm::{
    beacon::BeaconHeaderProof,
    execution::{AccountProof, ExecutionHeaderProof},
    EvmProofs,
};
use bankai_types::inputs::ProofBundle;
use bankai_types::results::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::blocks::parse_block_proof_payload;
use crate::fetch::api::ApiClient;
use crate::fetch::evm::{execution::ExecutionChainFetcher, op_stack::OpStackChainFetcher};
use crate::fetch::requests::{
    AccountProofRequest, BeaconHeaderProofRequest, EvmProofsRequest, ExecutionHeaderProofRequest,
    OpStackAccountProofRequest, OpStackHeaderProofRequest, OpStackProofsRequest,
    OpStackReceiptProofRequest, OpStackStorageSlotProofRequest, OpStackTxProofRequest,
    ReceiptProofRequest, StorageSlotProofRequest, TxProofRequest,
};
use crate::{Bankai, Network};

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

    pub fn ethereum_execution_header(mut self, block_number: u64) -> Self {
        let network_id = self.network.execution_network_id();
        let mut requests = self.ethereum.execution_header.take().unwrap_or_default();
        requests.push(ExecutionHeaderProofRequest {
            network_id,
            block_number,
        });
        self.ethereum.execution_header = Some(requests);
        self
    }

    pub fn ethereum_beacon_header(mut self, slot: u64) -> Self {
        let network_id = self.network.beacon_network_id();
        let mut requests = self.ethereum.beacon_header.take().unwrap_or_default();
        requests.push(BeaconHeaderProofRequest { network_id, slot });
        self.ethereum.beacon_header = Some(requests);
        self
    }

    pub fn ethereum_account(mut self, block_number: u64, address: Address) -> Self {
        let network_id = self.network.execution_network_id();
        let mut requests = self.ethereum.account.take().unwrap_or_default();
        requests.push(AccountProofRequest {
            network_id,
            block_number,
            address,
        });
        self.ethereum.account = Some(requests);
        self
    }

    pub fn ethereum_storage_slot(
        mut self,
        block_number: u64,
        address: Address,
        slot_keys: Vec<U256>,
    ) -> Self {
        let network_id = self.network.execution_network_id();
        let mut requests = self.ethereum.storage_slot.take().unwrap_or_default();
        requests.push(StorageSlotProofRequest {
            network_id,
            block_number,
            address,
            slot_keys,
        });
        self.ethereum.storage_slot = Some(requests);
        self
    }

    pub fn ethereum_tx(mut self, tx_hash: FixedBytes<32>) -> Self {
        let network_id = self.network.execution_network_id();
        let mut requests = self.ethereum.tx_proof.take().unwrap_or_default();
        requests.push(TxProofRequest {
            network_id,
            tx_hash,
        });
        self.ethereum.tx_proof = Some(requests);
        self
    }

    pub fn ethereum_receipt(mut self, tx_hash: FixedBytes<32>) -> Self {
        let network_id = self.network.execution_network_id();
        let mut requests = self.ethereum.receipt_proof.take().unwrap_or_default();
        requests.push(ReceiptProofRequest {
            network_id,
            tx_hash,
        });
        self.ethereum.receipt_proof = Some(requests);
        self
    }

    pub fn proof_format(mut self, proof_format: ProofFormat) -> Self {
        self.proof_format = proof_format;
        self
    }

    pub fn op_stack_header(mut self, chain_name: impl Into<String>, block_number: u64) -> Self {
        self.op_stack.header.push(OpStackHeaderProofRequest {
            chain_name: chain_name.into(),
            block_number: Some(block_number),
            header_hash: None,
        });
        self
    }

    pub fn op_stack_latest_header(mut self, chain_name: impl Into<String>) -> Self {
        self.op_stack.header.push(OpStackHeaderProofRequest {
            chain_name: chain_name.into(),
            block_number: None,
            header_hash: None,
        });
        self
    }

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

    pub fn op_stack_tx(mut self, chain_name: impl Into<String>, tx_hash: FixedBytes<32>) -> Self {
        self.op_stack.tx_proof.push(OpStackTxProofRequest {
            chain_name: chain_name.into(),
            tx_hash,
        });
        self
    }

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

    pub async fn execute(self) -> SdkResult<ProofBundle> {
        let api: &ApiClient = &self.bankai.api;
        let full_block = api.blocks().full(self.bankai_block_number).await?;
        let block = full_block.block.to_block();

        let mut exec_headers = BTreeSet::new();
        let mut beacon_headers = BTreeSet::new();

        if let Some(requests) = &self.ethereum.execution_header {
            for request in requests {
                exec_headers.insert((request.network_id, request.block_number));
            }
        }
        if let Some(requests) = &self.ethereum.beacon_header {
            for request in requests {
                beacon_headers.insert((request.network_id, request.slot));
            }
        }
        if let Some(requests) = &self.ethereum.account {
            for request in requests {
                exec_headers.insert((request.network_id, request.block_number));
            }
        }
        if let Some(requests) = &self.ethereum.storage_slot {
            for request in requests {
                exec_headers.insert((request.network_id, request.block_number));
            }
        }

        let mut tx_proofs = Vec::new();
        if let Some(requests) = &self.ethereum.tx_proof {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            for request in requests {
                if exec_fetcher.network_id() != request.network_id {
                    return Err(SdkError::InvalidInput(
                        "execution network_id mismatch".into(),
                    ));
                }
                tx_proofs.push(exec_fetcher.tx_proof(request.tx_hash).await?);
            }
        }

        let mut receipt_proofs = Vec::new();
        if let Some(requests) = &self.ethereum.receipt_proof {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            for request in requests {
                if exec_fetcher.network_id() != request.network_id {
                    return Err(SdkError::InvalidInput(
                        "execution network_id mismatch".into(),
                    ));
                }
                receipt_proofs.push(exec_fetcher.receipt_proof(request.tx_hash).await?);
            }
        }

        for proof in &tx_proofs {
            exec_headers.insert((proof.network_id, proof.block_number));
        }
        for proof in &receipt_proofs {
            exec_headers.insert((proof.network_id, proof.block_number));
        }

        let mut exec_header_map: BTreeMap<(u64, u64), ExecutionHeader> = BTreeMap::new();
        let mut beacon_header_map: BTreeMap<(u64, u64), HeaderResponse> = BTreeMap::new();

        for (network_id, block_number) in &exec_headers {
            let fetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            if fetcher.network_id() != *network_id {
                return Err(SdkError::InvalidInput(format!(
                    "execution network_id mismatch: requested {}, configured {}",
                    network_id,
                    fetcher.network_id()
                )));
            }
            exec_header_map.insert(
                (*network_id, *block_number),
                fetcher.header_only(*block_number).await?,
            );
        }

        for (network_id, slot) in &beacon_headers {
            let fetcher = self
                .bankai
                .ethereum()
                .beacon()
                .map_err(|_| SdkError::NotConfigured("Ethereum beacon fetcher".into()))?;
            if fetcher.network_id() != *network_id {
                return Err(SdkError::InvalidInput(format!(
                    "beacon network_id mismatch: requested {}, configured {}",
                    network_id,
                    fetcher.network_id()
                )));
            }
            beacon_header_map.insert((*network_id, *slot), fetcher.header_only(*slot).await?);
        }

        let filter = BankaiBlockFilterDto::with_bankai_block_number(self.bankai_block_number);
        let mut block_proof_value: Option<BlockProofPayloadDto> = None;
        let mut exec_mmr_by_hash: BTreeMap<String, _> = BTreeMap::new();
        let mut beacon_mmr_by_hash: BTreeMap<String, _> = BTreeMap::new();
        let mut op_snapshot_by_chain = BTreeMap::new();
        let mut op_header_map: BTreeMap<(String, String), ExecutionHeader> = BTreeMap::new();

        if !exec_header_map.is_empty() {
            let header_hashes: Vec<String> = exec_header_map
                .values()
                .map(|header| header.hash.to_string())
                .collect();
            let request = EthereumLightClientProofRequestDto {
                filter: filter.clone(),
                hashing_function: self.hashing,
                header_hashes,
                proof_format: self.proof_format,
            };
            let proof = api
                .ethereum()
                .execution()
                .light_client_proof(&request)
                .await?;
            if block_proof_value.is_none() {
                block_proof_value = Some(proof.block_proof.proof);
            }
            for mmr_proof in proof.mmr_proofs {
                exec_mmr_by_hash.insert(mmr_proof.header_hash.clone(), mmr_proof);
            }
        }

        if !beacon_header_map.is_empty() {
            let header_hashes: Vec<String> = beacon_header_map
                .values()
                .map(|header| {
                    let root = BeaconHeader::from(header.clone()).tree_hash_root();
                    format!("0x{}", root.encode_hex())
                })
                .collect();
            let request = EthereumLightClientProofRequestDto {
                filter: filter.clone(),
                hashing_function: self.hashing,
                header_hashes,
                proof_format: self.proof_format,
            };
            let proof = api.ethereum().beacon().light_client_proof(&request).await?;
            if block_proof_value.is_none() {
                block_proof_value = Some(proof.block_proof.proof);
            }
            for mmr_proof in proof.mmr_proofs {
                beacon_mmr_by_hash.insert(mmr_proof.header_hash.clone(), mmr_proof);
            }
        }

        for request in &self.op_stack.header {
            let fetcher = self.bankai.op_stack(&request.chain_name)?;
            let snapshot =
                get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone())
                    .await?;
            let header = match (request.block_number, request.header_hash) {
                (Some(block_number), _) => fetcher.header_only(block_number).await?,
                (None, Some(header_hash)) => fetcher.header_only_by_hash(header_hash).await?,
                (None, None) => {
                    let header_hash = parse_fixed_bytes(&snapshot.header_hash)?;
                    fetcher.header_only_by_hash(header_hash).await?
                }
            };
            op_header_map.insert(
                (request.chain_name.clone(), header.hash.to_string()),
                header,
            );
        }

        let mut op_account_proofs = Vec::new();
        for request in &self.op_stack.account {
            let fetcher = self.bankai.op_stack(&request.chain_name)?;
            let snapshot =
                get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone())
                    .await?;
            let header = fetcher.header_only(request.block_number).await?;
            op_header_map.insert(
                (request.chain_name.clone(), header.hash.to_string()),
                header.clone(),
            );
            let proof = fetcher
                .account(request.block_number, request.address)
                .await?;
            op_account_proofs.push(AccountProof {
                account: AlloyAccount {
                    balance: proof.balance,
                    nonce: proof.nonce,
                    code_hash: proof.code_hash,
                    storage_root: proof.storage_hash,
                },
                address: request.address,
                network_id: snapshot.chain_id,
                block_number: request.block_number,
                state_root: header.state_root,
                mpt_proof: proof.account_proof,
            });
        }

        let mut op_storage_slot_proofs = Vec::new();
        for request in &self.op_stack.storage_slot {
            let fetcher = self.bankai.op_stack(&request.chain_name)?;
            let _snapshot =
                get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone())
                    .await?;
            let header = fetcher.header_only(request.block_number).await?;
            op_header_map.insert(
                (request.chain_name.clone(), header.hash.to_string()),
                header,
            );
            op_storage_slot_proofs.push(
                fetcher
                    .storage_slot_proof(request.block_number, request.address, &request.slot_keys)
                    .await?,
            );
        }

        let mut op_tx_proofs = Vec::new();
        for request in &self.op_stack.tx_proof {
            let fetcher = self.bankai.op_stack(&request.chain_name)?;
            let snapshot =
                get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone())
                    .await?;
            let proof = fetcher.tx_proof(request.tx_hash).await?;
            if proof.network_id != snapshot.chain_id {
                return Err(SdkError::InvalidInput(format!(
                    "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
                    request.chain_name, proof.network_id, snapshot.chain_id
                )));
            }
            let header = fetcher.header_only(proof.block_number).await?;
            op_header_map.insert(
                (request.chain_name.clone(), header.hash.to_string()),
                header,
            );
            op_tx_proofs.push(proof);
        }

        let mut op_receipt_proofs = Vec::new();
        for request in &self.op_stack.receipt_proof {
            let fetcher = self.bankai.op_stack(&request.chain_name)?;
            let snapshot =
                get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone())
                    .await?;
            let proof = fetcher.receipt_proof(request.tx_hash).await?;
            if proof.network_id != snapshot.chain_id {
                return Err(SdkError::InvalidInput(format!(
                    "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
                    request.chain_name, proof.network_id, snapshot.chain_id
                )));
            }
            let header = fetcher.header_only(proof.block_number).await?;
            op_header_map.insert(
                (request.chain_name.clone(), header.hash.to_string()),
                header,
            );
            op_receipt_proofs.push(proof);
        }

        let mut op_stack_header_proofs = Vec::new();
        let mut op_header_hashes_by_chain: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for ((chain_name, header_hash), _) in &op_header_map {
            op_header_hashes_by_chain
                .entry(chain_name.clone())
                .or_default()
                .push(header_hash.clone());
        }
        for (chain_name, header_hashes) in op_header_hashes_by_chain {
            let snapshot = op_snapshot_by_chain
                .get(&chain_name)
                .cloned()
                .ok_or_else(|| {
                    SdkError::NotFound(format!("missing OP snapshot for {chain_name}"))
                })?;
            let request = OpStackLightClientProofRequestDto {
                filter: filter.clone(),
                hashing_function: self.hashing,
                header_hashes: header_hashes.clone(),
                proof_format: self.proof_format,
            };
            let proof = api
                .op_stack()
                .light_client_proof(&chain_name, &request)
                .await?;
            if block_proof_value.is_none() {
                block_proof_value = Some(proof.block_proof.proof.clone());
            }
            let mut mmr_by_hash = BTreeMap::new();
            for mmr_proof in proof.mmr_proofs {
                mmr_by_hash.insert(mmr_proof.header_hash.clone(), mmr_proof);
            }

            for header_hash in header_hashes {
                let header = op_header_map
                    .get(&(chain_name.clone(), header_hash.clone()))
                    .ok_or_else(|| {
                        SdkError::NotFound(format!(
                            "missing OP header for chain {} and hash {}",
                            chain_name, header_hash
                        ))
                    })?;
                let mmr_proof = mmr_by_hash.get(&header_hash).ok_or_else(|| {
                    SdkError::NotFound(format!(
                        "missing OP MMR proof for chain {} and hash {}",
                        chain_name, header_hash
                    ))
                })?;
                op_stack_header_proofs.push(OpStackHeaderProof {
                    header: header.clone(),
                    snapshot: op_snapshot_to_witness(snapshot.clone())?,
                    merkle_proof: proof.merkle_proof.clone().into(),
                    mmr_proof: mmr_proof.clone().into(),
                });
            }
        }

        let block_proof_value = match block_proof_value {
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

        let mut exec_header_proofs = Vec::new();
        for header in exec_header_map.values() {
            let mmr_proof = exec_mmr_by_hash
                .get(&header.hash.to_string())
                .ok_or_else(|| {
                    SdkError::NotFound("missing MMR proof for execution header".into())
                })?;
            exec_header_proofs.push(ExecutionHeaderProof {
                header: header.clone(),
                mmr_proof: mmr_proof.clone().into(),
            });
        }

        let mut beacon_header_proofs = Vec::new();
        for header in beacon_header_map.values() {
            let root = BeaconHeader::from(header.clone()).tree_hash_root();
            let key = format!("0x{}", root.encode_hex());
            let mmr_proof = beacon_mmr_by_hash
                .get(&key)
                .ok_or_else(|| SdkError::NotFound("missing MMR proof for beacon header".into()))?;
            beacon_header_proofs.push(BeaconHeaderProof {
                header: header.clone(),
                mmr_proof: mmr_proof.clone().into(),
            });
        }

        let mut account_proofs = Vec::new();
        if let Some(requests) = &self.ethereum.account {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            for request in requests {
                if exec_fetcher.network_id() != request.network_id {
                    return Err(SdkError::InvalidInput(
                        "execution network_id mismatch".into(),
                    ));
                }
                let proof = exec_fetcher
                    .account(
                        request.block_number,
                        request.address,
                        self.hashing,
                        self.bankai_block_number,
                    )
                    .await?;
                let header = exec_header_map
                    .get(&(request.network_id, request.block_number))
                    .ok_or_else(|| SdkError::NotFound("header not fetched for account".into()))?;
                account_proofs.push(AccountProof {
                    account: AlloyAccount {
                        balance: proof.balance,
                        nonce: proof.nonce,
                        code_hash: proof.code_hash,
                        storage_root: proof.storage_hash,
                    },
                    address: request.address,
                    network_id: request.network_id,
                    block_number: request.block_number,
                    state_root: header.state_root,
                    mpt_proof: proof.account_proof,
                });
            }
        }

        let mut storage_slot_proofs = Vec::new();
        if let Some(requests) = &self.ethereum.storage_slot {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            for request in requests {
                if exec_fetcher.network_id() != request.network_id {
                    return Err(SdkError::InvalidInput(
                        "execution network_id mismatch".into(),
                    ));
                }
                storage_slot_proofs.push(
                    exec_fetcher
                        .storage_slot_proof(
                            request.block_number,
                            request.address,
                            &request.slot_keys,
                            self.hashing,
                            self.bankai_block_number,
                        )
                        .await?,
                );
            }
        }

        let evm_proofs = EvmProofs {
            execution_header_proof: option_vec(exec_header_proofs),
            beacon_header_proof: option_vec(beacon_header_proofs),
            account_proof: option_vec(account_proofs),
            storage_slot_proof: option_vec(storage_slot_proofs),
            tx_proof: option_vec(tx_proofs),
            receipt_proof: option_vec(receipt_proofs),
        };
        let op_stack_proofs = OpStackProofs {
            header_proof: option_vec(op_stack_header_proofs),
            account_proof: option_vec(op_account_proofs),
            storage_slot_proof: option_vec(op_storage_slot_proofs),
            tx_proof: option_vec(op_tx_proofs),
            receipt_proof: option_vec(op_receipt_proofs),
        };

        Ok(ProofBundle {
            hashing_function: self.hashing,
            block_proof,
            block,
            evm_proofs: if evm_proofs.execution_header_proof.is_none()
                && evm_proofs.beacon_header_proof.is_none()
                && evm_proofs.account_proof.is_none()
                && evm_proofs.storage_slot_proof.is_none()
                && evm_proofs.tx_proof.is_none()
                && evm_proofs.receipt_proof.is_none()
            {
                None
            } else {
                Some(evm_proofs)
            },
            op_stack_proofs: if op_stack_proofs.header_proof.is_none()
                && op_stack_proofs.account_proof.is_none()
                && op_stack_proofs.storage_slot_proof.is_none()
                && op_stack_proofs.tx_proof.is_none()
                && op_stack_proofs.receipt_proof.is_none()
            {
                None
            } else {
                Some(op_stack_proofs)
            },
        })
    }
}

fn option_vec<T>(items: Vec<T>) -> Option<Vec<T>> {
    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

async fn get_or_fetch_op_snapshot(
    snapshots: &mut BTreeMap<String, bankai_types::api::op_stack::OpChainSnapshotSummaryDto>,
    fetcher: &OpStackChainFetcher,
    filter: BankaiBlockFilterDto,
) -> SdkResult<bankai_types::api::op_stack::OpChainSnapshotSummaryDto> {
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

fn op_snapshot_to_witness(
    snapshot: bankai_types::api::op_stack::OpChainSnapshotSummaryDto,
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

fn parse_fixed_bytes(value: &str) -> SdkResult<FixedBytes<32>> {
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
