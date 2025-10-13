use std::collections::{BTreeMap, BTreeSet};

use alloy_primitives::{hex::ToHexExt, Address};
use alloy_rpc_types::{Account as AlloyAccount, Header as ExecutionHeader};

use bankai_types::api::proofs::HashingFunctionDto;
use bankai_types::api::proofs::{HeaderRequestDto, LightClientProofRequestDto};
use bankai_types::fetch::evm::{
    beacon::BeaconHeaderProof,
    execution::{AccountProof, ExecutionHeaderProof},
    AccountProofRequest, BeaconHeaderProofRequest, EvmProofs, EvmProofsRequest,
    ExecutionHeaderProofRequest,
};
use bankai_types::fetch::ProofWrapper;
use bankai_types::verify::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use crate::errors::{SdkError, SdkResult};
use crate::fetch::bankai::stwo::parse_block_proof_value;
use crate::fetch::clients::bankai_api::ApiClient;
use crate::fetch::evm::execution::ExecutionChainFetcher;
use crate::Bankai;

pub struct ProofBatchBuilder<'a> {
    bankai: &'a Bankai,
    bankai_block_number: u64,
    hashing: HashingFunctionDto,
    evm: EvmProofsRequest,
}

impl<'a> ProofBatchBuilder<'a> {
    pub fn new(bankai: &'a Bankai, bankai_block_number: u64, hashing: HashingFunctionDto) -> Self {
        Self {
            bankai,
            bankai_block_number,
            hashing,
            evm: EvmProofsRequest {
                execution_header: None,
                beacon_header: None,
                account: None,
            },
        }
    }

    pub fn evm_execution_header(mut self, network_id: u64, block_number: u64) -> Self {
        let mut v = self.evm.execution_header.take().unwrap_or_default();
        v.push(ExecutionHeaderProofRequest {
            network_id,
            block_number,
            hashing_function: self.hashing.clone(),
            bankai_block_number: self.bankai_block_number,
        });
        self.evm.execution_header = Some(v);
        self
    }

    pub fn evm_beacon_header(mut self, network_id: u64, slot: u64) -> Self {
        let mut v = self.evm.beacon_header.take().unwrap_or_default();
        v.push(BeaconHeaderProofRequest {
            network_id,
            slot,
            hashing_function: self.hashing.clone(),
            bankai_block_number: self.bankai_block_number,
        });
        self.evm.beacon_header = Some(v);
        self
    }

    pub fn evm_account(mut self, network_id: u64, block_number: u64, address: Address) -> Self {
        let mut v = self.evm.account.take().unwrap_or_default();
        v.push(AccountProofRequest {
            network_id,
            block_number,
            address,
        });
        self.evm.account = Some(v);
        self
    }

    pub async fn execute(self) -> SdkResult<ProofWrapper> {
        // Build working sets
        let mut exec_headers: BTreeSet<(u64, u64)> = BTreeSet::new(); // (network_id, block_number)
        let mut beacon_headers: BTreeSet<(u64, u64)> = BTreeSet::new(); // (network_id, slot)

        if let Some(explicit) = &self.evm.execution_header {
            for r in explicit {
                exec_headers.insert((r.network_id, r.block_number));
            }
        }
        if let Some(bh) = &self.evm.beacon_header {
            for r in bh {
                beacon_headers.insert((r.network_id, r.slot));
            }
        }
        if let Some(accounts) = &self.evm.account {
            for r in accounts {
                exec_headers.insert((r.network_id, r.block_number));
            }
        }

        // Fetch headers via RPC only
        let mut exec_header_map: BTreeMap<(u64, u64), ExecutionHeader> = BTreeMap::new();
        let mut beacon_header_map: BTreeMap<(u64, u64), BeaconHeader> = BTreeMap::new();

        for (network_id, block_number) in &exec_headers {
            let fetcher = self
                .bankai
                .evm
                .execution()
                .map_err(|_| SdkError::NotConfigured("EVM execution fetcher".into()))?;
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
                .evm
                .beacon()
                .map_err(|_| SdkError::NotConfigured("EVM beacon fetcher".into()))?;
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

        // Build light-client request
        let mut requested_headers: Vec<HeaderRequestDto> = Vec::new();
        for ((network_id, _), header) in exec_header_map.iter() {
            requested_headers.push(HeaderRequestDto {
                network_id: *network_id,
                header_hash: header.hash.to_string(),
            });
        }
        for ((network_id, _), header) in beacon_header_map.iter() {
            let root = header.tree_hash_root();
            requested_headers.push(HeaderRequestDto {
                network_id: *network_id,
                header_hash: format!("0x{}", root.encode_hex()),
            });
        }

        // Single light-client call
        let api: &ApiClient = &self.bankai.api;
        let lc_req = LightClientProofRequestDto {
            bankai_block_number: Some(self.bankai_block_number),
            hashing_function: self.hashing.clone(),
            requested_headers,
        };
        let lc_proof = api.get_light_client_proof(&lc_req).await?;

        // Parse block proof
        let block_proof = parse_block_proof_value(lc_proof.block_proof.proof);
        let block_proof = block_proof?;

        // Index MMR proofs
        let mut mmr_by_key: BTreeMap<(u64, String), _> = BTreeMap::new();
        for p in lc_proof.mmr_proofs {
            mmr_by_key.insert((p.network_id, p.header_hash.clone()), p);
        }

        // Build header proofs
        let mut exec_header_proofs: Vec<ExecutionHeaderProof> = Vec::new();
        for ((network_id, _), header) in exec_header_map.iter() {
            let key = (*network_id, header.hash.to_string());
            let mmr = mmr_by_key.get(&key).ok_or_else(|| {
                SdkError::NotFound("missing MMR proof for execution header".into())
            })?;
            exec_header_proofs.push(ExecutionHeaderProof {
                header: header.clone(),
                mmr_proof: mmr.clone(),
            });
        }

        let mut beacon_header_proofs: Vec<BeaconHeaderProof> = Vec::new();
        for ((network_id, _), header) in beacon_header_map.iter() {
            let root = header.tree_hash_root();
            let key = (*network_id, format!("0x{}", root.encode_hex()));
            let mmr = mmr_by_key
                .get(&key)
                .ok_or_else(|| SdkError::NotFound("missing MMR proof for beacon header".into()))?;
            beacon_header_proofs.push(BeaconHeaderProof {
                header: header.clone(),
                mmr_proof: mmr.clone(),
            });
        }

        // Fetch account proofs
        let mut account_proofs: Vec<AccountProof> = Vec::new();
        if let Some(accounts) = &self.evm.account {
            let exec_fetcher: &ExecutionChainFetcher = self
                .bankai
                .evm
                .execution()
                .map_err(|_| SdkError::NotConfigured("EVM execution fetcher".into()))?;
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
                        self.hashing.clone(),
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
        };

        let wrapper = ProofWrapper {
            block_proof,
            hashing_function: self.hashing.clone(),
            evm_proofs: Some(evm_proofs),
        };
        Ok(wrapper)
    }
}
