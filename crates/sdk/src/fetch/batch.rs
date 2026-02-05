use std::collections::{BTreeMap, BTreeSet};

use alloy_primitives::FixedBytes;
use alloy_primitives::{hex::ToHexExt, Address, U256};
use alloy_rpc_types_eth::{Account as AlloyAccount, Header as ExecutionHeader};

use bankai_types::api::ethereum::{BankaiBlockFilterDto, EthereumLightClientProofRequestDto};
use bankai_types::api::proofs::HashingFunctionDto;
use bankai_types::fetch::evm::execution::{StorageSlotProof, TxProof};
use bankai_types::fetch::evm::TxProofRequest;
use bankai_types::fetch::evm::{
    beacon::BeaconHeaderProof,
    execution::{AccountProof, ExecutionHeaderProof},
    AccountProofRequest, BeaconHeaderProofRequest, EvmProofs, EvmProofsRequest,
    ExecutionHeaderProofRequest, StorageSlotProofRequest,
};
use bankai_types::fetch::ProofBundle;
use bankai_types::verify::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::blocks::parse_block_proof_value;
use crate::fetch::api::ApiClient;
use crate::fetch::ethereum::execution::ExecutionChainFetcher;
use crate::{Bankai, Network};

pub struct ProofBatchBuilder<'a> {
    bankai: &'a Bankai,
    network: Network,
    bankai_block_number: u64,
    hashing: HashingFunctionDto,
    ethereum: EvmProofsRequest,
}

impl<'a> ProofBatchBuilder<'a> {
    pub fn new(
        bankai: &'a Bankai,
        network: Network,
        bankai_block_number: u64,
        hashing: HashingFunctionDto,
    ) -> Self {
        Self {
            bankai,
            network,
            bankai_block_number,
            hashing,
            ethereum: EvmProofsRequest {
                execution_header: None,
                beacon_header: None,
                account: None,
                storage_slot: None,
                tx_proof: None,
            },
        }
    }

    /// Adds an execution header to the batch
    ///
    /// # Arguments
    ///
    /// * `block_number` - The execution layer block number to fetch
    pub fn ethereum_execution_header(mut self, block_number: u64) -> Self {
        let network_id = self.network.execution_network_id();
        let mut v = self.ethereum.execution_header.take().unwrap_or_default();
        v.push(ExecutionHeaderProofRequest {
            network_id,
            block_number,
            hashing_function: self.hashing,
            bankai_block_number: self.bankai_block_number,
        });
        self.ethereum.execution_header = Some(v);
        self
    }

    /// Adds a beacon header to the batch
    ///
    /// # Arguments
    ///
    /// * `slot` - The beacon chain slot number to fetch
    pub fn ethereum_beacon_header(mut self, slot: u64) -> Self {
        let network_id = self.network.beacon_network_id();
        let mut v = self.ethereum.beacon_header.take().unwrap_or_default();
        v.push(BeaconHeaderProofRequest {
            network_id,
            slot,
            hashing_function: self.hashing,
            bankai_block_number: self.bankai_block_number,
        });
        self.ethereum.beacon_header = Some(v);
        self
    }

    /// Adds an account proof to the batch
    ///
    /// # Arguments
    ///
    /// * `block_number` - The execution layer block number to query
    /// * `address` - The account address to fetch proof for
    pub fn ethereum_account(mut self, block_number: u64, address: Address) -> Self {
        let network_id = self.network.execution_network_id();
        let mut v = self.ethereum.account.take().unwrap_or_default();
        v.push(AccountProofRequest {
            network_id,
            block_number,
            address,
        });
        self.ethereum.account = Some(v);
        self
    }

    /// Adds a storage slot proof to the batch for one or more slots from the same contract.
    ///
    /// # Arguments
    ///
    /// * `block_number` - The execution layer block number to query
    /// * `address` - The contract address
    /// * `slot_keys` - The storage slot keys (uint256) to query
    pub fn ethereum_storage_slot(
        mut self,
        block_number: u64,
        address: Address,
        slot_keys: Vec<U256>,
    ) -> Self {
        let network_id = self.network.execution_network_id();
        let mut v = self.ethereum.storage_slot.take().unwrap_or_default();
        v.push(StorageSlotProofRequest {
            network_id,
            block_number,
            address,
            slot_keys,
        });
        self.ethereum.storage_slot = Some(v);
        self
    }

    /// Adds a transaction proof to the batch
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The transaction hash to fetch proof for
    pub fn ethereum_tx(mut self, tx_hash: FixedBytes<32>) -> Self {
        let network_id = self.network.execution_network_id();
        let mut v = self.ethereum.tx_proof.take().unwrap_or_default();
        v.push(TxProofRequest {
            network_id,
            tx_hash,
        });
        self.ethereum.tx_proof = Some(v);
        self
    }

    pub async fn execute(self) -> SdkResult<ProofBundle> {
        // Build working sets
        let mut exec_headers: BTreeSet<(u64, u64)> = BTreeSet::new(); // (network_id, block_number)
        let mut beacon_headers: BTreeSet<(u64, u64)> = BTreeSet::new(); // (network_id, slot)

        if let Some(explicit) = &self.ethereum.execution_header {
            for r in explicit {
                exec_headers.insert((r.network_id, r.block_number));
            }
        }
        if let Some(bh) = &self.ethereum.beacon_header {
            for r in bh {
                beacon_headers.insert((r.network_id, r.slot));
            }
        }
        if let Some(accounts) = &self.ethereum.account {
            for r in accounts {
                exec_headers.insert((r.network_id, r.block_number));
            }
        }
        if let Some(slots) = &self.ethereum.storage_slot {
            for r in slots {
                exec_headers.insert((r.network_id, r.block_number));
            }
        }

        let mut tx_proofs: Vec<TxProof> = Vec::new();
        if let Some(txs) = &self.ethereum.tx_proof {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            for req in txs {
                if exec_fetcher.network_id() != req.network_id {
                    return Err(SdkError::InvalidInput(
                        "execution network_id mismatch".into(),
                    ));
                }
                let proof = exec_fetcher.tx_proof(req.tx_hash).await?;
                tx_proofs.push(proof);
            }
        };

        // add tx proofs to exec_headers
        for proof in tx_proofs.clone() {
            exec_headers.insert((proof.network_id, proof.block_number));
        }

        // Fetch headers via RPC only
        let mut exec_header_map: BTreeMap<(u64, u64), ExecutionHeader> = BTreeMap::new();
        let mut beacon_header_map: BTreeMap<(u64, u64), BeaconHeader> = BTreeMap::new();

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
            let header = fetcher.header_only(*block_number).await?;
            exec_header_map.insert((*network_id, *block_number), header);
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
            let header = fetcher.header_only(*slot).await?;
            beacon_header_map.insert((*network_id, *slot), header);
        }

        // Build light-client requests (execution + beacon)
        let filter = BankaiBlockFilterDto::with_bankai_block_number(self.bankai_block_number);
        let api: &ApiClient = &self.bankai.api;
        let mut block_proof_value: Option<serde_json::Value> = None;
        let mut exec_mmr_by_hash: BTreeMap<String, _> = BTreeMap::new();
        let mut beacon_mmr_by_hash: BTreeMap<String, _> = BTreeMap::new();

        if !exec_header_map.is_empty() {
            let header_hashes: Vec<String> = exec_header_map
                .values()
                .map(|header| header.hash.to_string())
                .collect();
            let lc_req = EthereumLightClientProofRequestDto {
                filter: filter.clone(),
                hashing_function: self.hashing,
                header_hashes,
            };
            let lc_proof = api
                .ethereum()
                .execution()
                .light_client_proof(&lc_req)
                .await?;
            if block_proof_value.is_none() {
                block_proof_value = Some(lc_proof.block_proof.proof);
            }
            for p in lc_proof.mmr_proofs {
                exec_mmr_by_hash.insert(p.header_hash.clone(), p);
            }
        }

        if !beacon_header_map.is_empty() {
            let header_hashes: Vec<String> = beacon_header_map
                .values()
                .map(|header| {
                    let root = header.tree_hash_root();
                    format!("0x{}", root.encode_hex())
                })
                .collect();
            let lc_req = EthereumLightClientProofRequestDto {
                filter: filter.clone(),
                hashing_function: self.hashing,
                header_hashes,
            };
            let lc_proof = api.ethereum().beacon().light_client_proof(&lc_req).await?;
            if block_proof_value.is_none() {
                block_proof_value = Some(lc_proof.block_proof.proof);
            }
            for p in lc_proof.mmr_proofs {
                beacon_mmr_by_hash.insert(p.header_hash.clone(), p);
            }
        }

        let block_proof_value = match block_proof_value {
            Some(value) => value,
            None => api.blocks().proof(self.bankai_block_number).await?.proof,
        };
        let block_proof = parse_block_proof_value(block_proof_value)?;

        // Build header proofs
        let mut exec_header_proofs: Vec<ExecutionHeaderProof> = Vec::new();
        for ((_, _), header) in exec_header_map.iter() {
            let key = header.hash.to_string();
            let mmr = exec_mmr_by_hash.get(&key).ok_or_else(|| {
                SdkError::NotFound("missing MMR proof for execution header".into())
            })?;
            exec_header_proofs.push(ExecutionHeaderProof {
                header: header.clone(),
                mmr_proof: mmr.clone().into(),
            });
        }

        let mut beacon_header_proofs: Vec<BeaconHeaderProof> = Vec::new();
        for ((_, _), header) in beacon_header_map.iter() {
            let root = header.tree_hash_root();
            let key = format!("0x{}", root.encode_hex());
            let mmr = beacon_mmr_by_hash
                .get(&key)
                .ok_or_else(|| SdkError::NotFound("missing MMR proof for beacon header".into()))?;
            beacon_header_proofs.push(BeaconHeaderProof {
                header: header.clone(),
                mmr_proof: mmr.clone().into(),
            });
        }

        // Fetch account proofs
        let mut account_proofs: Vec<AccountProof> = Vec::new();
        if let Some(accounts) = &self.ethereum.account {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            for req in accounts {
                if exec_fetcher.network_id() != req.network_id {
                    return Err(SdkError::InvalidInput(
                        "execution network_id mismatch".into(),
                    ));
                }
                let proof = exec_fetcher
                    .account(
                        req.block_number,
                        req.address,
                        self.hashing,
                        self.bankai_block_number,
                    )
                    .await?;
                let header = exec_header_map
                    .get(&(req.network_id, req.block_number))
                    .ok_or_else(|| SdkError::NotFound("header not fetched for account".into()))?;
                let account_state: AlloyAccount = AlloyAccount {
                    balance: proof.balance,
                    nonce: proof.nonce,
                    code_hash: proof.code_hash,
                    storage_root: proof.storage_hash,
                };
                let account_proof = AccountProof {
                    account: account_state,
                    address: req.address,
                    network_id: req.network_id,
                    block_number: req.block_number,
                    state_root: header.state_root,
                    mpt_proof: proof.account_proof,
                };
                account_proofs.push(account_proof);
            }
        }

        // Fetch storage slot proofs
        let mut storage_slot_proofs: Vec<StorageSlotProof> = Vec::new();
        if let Some(slots) = &self.ethereum.storage_slot {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .ethereum()
                .execution()
                .map_err(|_| SdkError::NotConfigured("Ethereum execution fetcher".into()))?;
            for req in slots {
                if exec_fetcher.network_id() != req.network_id {
                    return Err(SdkError::InvalidInput(
                        "execution network_id mismatch".into(),
                    ));
                }
                let proof = exec_fetcher
                    .storage_slot_proof(
                        req.block_number,
                        req.address,
                        &req.slot_keys,
                        self.hashing,
                        self.bankai_block_number,
                    )
                    .await?;
                storage_slot_proofs.push(proof);
            }
        }

        let evm_proofs = EvmProofs {
            execution_header_proof: if exec_header_proofs.is_empty() {
                None
            } else {
                Some(exec_header_proofs)
            },
            beacon_header_proof: if beacon_header_proofs.is_empty() {
                None
            } else {
                Some(beacon_header_proofs)
            },
            account_proof: if account_proofs.is_empty() {
                None
            } else {
                Some(account_proofs)
            },
            storage_slot_proof: if storage_slot_proofs.is_empty() {
                None
            } else {
                Some(storage_slot_proofs)
            },
            tx_proof: if tx_proofs.is_empty() {
                None
            } else {
                Some(tx_proofs)
            },
        };

        let wrapper = ProofBundle {
            block_proof,
            hashing_function: self.hashing,
            evm_proofs: Some(evm_proofs),
        };
        Ok(wrapper)
    }
}
