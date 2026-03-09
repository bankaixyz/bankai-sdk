use std::collections::{BTreeMap, BTreeSet};

use alloy_primitives::FixedBytes;
use alloy_primitives::{hex::ToHexExt, Address, U256};
use alloy_rpc_types_beacon::header::HeaderResponse;
use alloy_rpc_types_eth::{Account as AlloyAccount, Header as ExecutionHeader};
use bankai_types::api::ethereum::{BankaiBlockFilterDto, EthereumLightClientProofRequestDto};
use bankai_types::api::op_stack::OpStackLightClientProofRequestDto;
use bankai_types::api::proofs::BlockProofPayloadDto;
use bankai_types::common::{HashingFunction, ProofFormat};
use bankai_types::inputs::evm::{
    beacon::BeaconHeaderProof,
    execution::{AccountProof, ExecutionHeaderProof},
    EvmProofs,
};
use bankai_types::inputs::op_stack::{OpStackHeaderProof, OpStackProofs};
use bankai_types::inputs::ProofBundle;
use bankai_types::results::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::blocks::parse_block_proof_payload;
use crate::fetch::api::ApiClient;
use crate::fetch::ethereum::execution::ExecutionChainFetcher;
use crate::fetch::requests::{
    AccountProofRequest, BeaconHeaderProofRequest, EvmProofsRequest, ExecutionHeaderProofRequest,
    OpStackHeaderProofRequest, StorageSlotProofRequest, TxProofRequest,
};
use crate::{Bankai, Network};

pub struct ProofBatchBuilder<'a> {
    bankai: &'a Bankai,
    network: Network,
    bankai_block_number: u64,
    hashing: HashingFunction,
    proof_format: ProofFormat,
    ethereum: EvmProofsRequest,
    op_stack: Vec<OpStackHeaderProofRequest>,
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
            op_stack: Vec::new(),
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
        requests.push(BeaconHeaderProofRequest {
            network_id,
            slot,
        });
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

    pub fn proof_format(mut self, proof_format: ProofFormat) -> Self {
        self.proof_format = proof_format;
        self
    }

    pub fn op_stack_header(mut self, chain_name: impl Into<String>) -> Self {
        self.op_stack.push(OpStackHeaderProofRequest {
            chain_name: chain_name.into(),
            header_hash: None,
        });
        self
    }

    pub fn op_stack_header_by_hash(
        mut self,
        chain_name: impl Into<String>,
        header_hash: FixedBytes<32>,
    ) -> Self {
        self.op_stack.push(OpStackHeaderProofRequest {
            chain_name: chain_name.into(),
            header_hash: Some(header_hash),
        });
        self
    }

    pub async fn execute(self) -> SdkResult<ProofBundle> {
        let api: &ApiClient = &self.bankai.api;
        let full_block = api.blocks().full(self.bankai_block_number).await?;
        let block = full_block.block.to_block();
        let op_snapshots_by_chain_id = full_block
            .block
            .op_chains
            .iter()
            .cloned()
            .map(|snapshot| (snapshot.chain_id, snapshot))
            .collect::<BTreeMap<_, _>>();

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
                    return Err(SdkError::InvalidInput("execution network_id mismatch".into()));
                }
                tx_proofs.push(exec_fetcher.tx_proof(request.tx_hash).await?);
            }
        }

        for proof in &tx_proofs {
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
            exec_header_map.insert((*network_id, *block_number), fetcher.header_only(*block_number).await?);
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
        let mut op_stack_header_proofs = Vec::new();

        if !exec_header_map.is_empty() {
            let header_hashes: Vec<String> =
                exec_header_map.values().map(|header| header.hash.to_string()).collect();
            let request = EthereumLightClientProofRequestDto {
                filter: filter.clone(),
                hashing_function: self.hashing,
                header_hashes,
                proof_format: self.proof_format,
            };
            let proof = api.ethereum().execution().light_client_proof(&request).await?;
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

        for request in &self.op_stack {
            let snapshot = api.op_stack().snapshot(&request.chain_name, &filter).await?;
            let header_hash = request
                .header_hash
                .map(|hash| hash.to_string())
                .unwrap_or_else(|| snapshot.header_hash.clone());
            let light_client_request = OpStackLightClientProofRequestDto {
                filter: filter.clone(),
                hashing_function: self.hashing,
                header_hashes: vec![header_hash],
                proof_format: self.proof_format,
            };
            let proof = api
                .op_stack()
                .light_client_proof(&request.chain_name, &light_client_request)
                .await?;
            if block_proof_value.is_none() {
                block_proof_value = Some(proof.block_proof.proof.clone());
            }

            let snapshot_witness = op_snapshots_by_chain_id
                .get(&proof.merkle_proof.chain_id)
                .cloned()
                .ok_or_else(|| {
                    SdkError::NotFound(format!(
                        "missing OP snapshot witness for chain_id {}",
                        proof.merkle_proof.chain_id
                    ))
                })?;
            let mmr_proof = proof.mmr_proofs.into_iter().next().ok_or_else(|| {
                SdkError::NotFound(format!(
                    "missing OP MMR proof for chain {}",
                    request.chain_name
                ))
            })?;

            op_stack_header_proofs.push(OpStackHeaderProof {
                snapshot: snapshot_witness,
                merkle_proof: proof.merkle_proof.into(),
                mmr_proof: mmr_proof.into(),
            });
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
                .ok_or_else(|| SdkError::NotFound("missing MMR proof for execution header".into()))?;
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
                    return Err(SdkError::InvalidInput("execution network_id mismatch".into()));
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
                    return Err(SdkError::InvalidInput("execution network_id mismatch".into()));
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
            {
                None
            } else {
                Some(evm_proofs)
            },
            op_stack_proofs: option_vec(op_stack_header_proofs)
                .map(|header_proofs| OpStackProofs { header_proofs }),
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
