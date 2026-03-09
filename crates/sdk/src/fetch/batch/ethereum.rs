use std::collections::{BTreeMap, BTreeSet};

use alloy_primitives::hex::ToHexExt;
use alloy_rpc_types_beacon::header::HeaderResponse;
use alloy_rpc_types_eth::{Account as AlloyAccount, Header as ExecutionHeader};
use bankai_types::api::ethereum::{BankaiBlockFilterDto, EthereumLightClientProofRequestDto};
use bankai_types::api::proofs::BlockProofPayloadDto;
use bankai_types::inputs::evm::beacon::BeaconHeaderProof;
use bankai_types::inputs::evm::execution::{
    AccountProof, ExecutionHeaderProof, ReceiptProof, StorageSlotProof, TxProof,
};
use bankai_types::results::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use super::{beacon_fetcher, execution_fetcher, ProofBatchBuilder};
use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::ApiClient;

pub(super) struct EthereumBatchData {
    pub block_proof_value: Option<BlockProofPayloadDto>,
    pub execution_header_proofs: Vec<ExecutionHeaderProof>,
    pub beacon_header_proofs: Vec<BeaconHeaderProof>,
    pub account_proofs: Vec<AccountProof>,
    pub storage_slot_proofs: Vec<StorageSlotProof>,
    pub tx_proofs: Vec<TxProof>,
    pub receipt_proofs: Vec<ReceiptProof>,
}

pub(super) async fn assemble_ethereum_proofs(
    builder: &ProofBatchBuilder<'_>,
    api: &ApiClient,
    filter: &BankaiBlockFilterDto,
) -> SdkResult<EthereumBatchData> {
    let mut exec_headers = BTreeSet::new();
    let mut beacon_headers = BTreeSet::new();

    for request in &builder.ethereum.execution_header {
        exec_headers.insert((request.network_id, request.block_number));
    }
    for request in &builder.ethereum.beacon_header {
        beacon_headers.insert((request.network_id, request.slot));
    }
    for request in &builder.ethereum.account {
        exec_headers.insert((request.network_id, request.block_number));
    }
    for request in &builder.ethereum.storage_slot {
        exec_headers.insert((request.network_id, request.block_number));
    }

    let exec_fetcher = execution_fetcher(builder)?;
    let mut tx_proofs = Vec::new();
    for request in &builder.ethereum.tx_proof {
        if exec_fetcher.network_id() != request.network_id {
            return Err(SdkError::InvalidInput(
                "execution network_id mismatch".into(),
            ));
        }
        tx_proofs.push(exec_fetcher.tx_proof(request.tx_hash).await?);
    }

    let mut receipt_proofs = Vec::new();
    for request in &builder.ethereum.receipt_proof {
        if exec_fetcher.network_id() != request.network_id {
            return Err(SdkError::InvalidInput(
                "execution network_id mismatch".into(),
            ));
        }
        receipt_proofs.push(exec_fetcher.receipt_proof(request.tx_hash).await?);
    }

    for proof in &tx_proofs {
        exec_headers.insert((proof.network_id, proof.block_number));
    }
    for proof in &receipt_proofs {
        exec_headers.insert((proof.network_id, proof.block_number));
    }

    let mut exec_header_map: BTreeMap<(u64, u64), ExecutionHeader> = BTreeMap::new();
    for (network_id, block_number) in &exec_headers {
        if exec_fetcher.network_id() != *network_id {
            return Err(SdkError::InvalidInput(format!(
                "execution network_id mismatch: requested {}, configured {}",
                network_id,
                exec_fetcher.network_id()
            )));
        }
        exec_header_map.insert(
            (*network_id, *block_number),
            exec_fetcher.header_only(*block_number).await?,
        );
    }

    let beacon_fetcher = beacon_fetcher(builder)?;
    let mut beacon_header_map: BTreeMap<(u64, u64), HeaderResponse> = BTreeMap::new();
    for (network_id, slot) in &beacon_headers {
        if beacon_fetcher.network_id() != *network_id {
            return Err(SdkError::InvalidInput(format!(
                "beacon network_id mismatch: requested {}, configured {}",
                network_id,
                beacon_fetcher.network_id()
            )));
        }
        beacon_header_map.insert((*network_id, *slot), beacon_fetcher.header_only(*slot).await?);
    }

    let mut block_proof_value = None;
    let mut exec_mmr_by_hash = BTreeMap::new();
    let mut beacon_mmr_by_hash = BTreeMap::new();

    if !exec_header_map.is_empty() {
        let header_hashes = exec_header_map
            .values()
            .map(|header| header.hash.to_string())
            .collect();
        let request = EthereumLightClientProofRequestDto {
            filter: filter.clone(),
            hashing_function: builder.hashing,
            header_hashes,
            proof_format: builder.proof_format,
        };
        let proof = api.ethereum().execution().light_client_proof(&request).await?;
        if block_proof_value.is_none() {
            block_proof_value = Some(proof.block_proof.proof.clone());
        }
        for mmr_proof in proof.mmr_proofs {
            exec_mmr_by_hash.insert(mmr_proof.header_hash.clone(), mmr_proof);
        }
    }

    if !beacon_header_map.is_empty() {
        let header_hashes = beacon_header_map
            .values()
            .map(|header| {
                let root = BeaconHeader::from(header.clone()).tree_hash_root();
                format!("0x{}", root.encode_hex())
            })
            .collect();
        let request = EthereumLightClientProofRequestDto {
            filter: filter.clone(),
            hashing_function: builder.hashing,
            header_hashes,
            proof_format: builder.proof_format,
        };
        let proof = api.ethereum().beacon().light_client_proof(&request).await?;
        if block_proof_value.is_none() {
            block_proof_value = Some(proof.block_proof.proof.clone());
        }
        for mmr_proof in proof.mmr_proofs {
            beacon_mmr_by_hash.insert(mmr_proof.header_hash.clone(), mmr_proof);
        }
    }

    let mut execution_header_proofs = Vec::new();
    for header in exec_header_map.values() {
        let mmr_proof = exec_mmr_by_hash
            .get(&header.hash.to_string())
            .ok_or_else(|| SdkError::NotFound("missing MMR proof for execution header".into()))?;
        execution_header_proofs.push(ExecutionHeaderProof {
            header: header.clone(),
            mmr_proof: mmr_proof.clone().try_into().map_err(|e| {
                SdkError::InvalidInput(format!("invalid execution MMR proof hex from API: {e}"))
            })?,
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
            mmr_proof: mmr_proof.clone().try_into().map_err(|e| {
                SdkError::InvalidInput(format!("invalid beacon MMR proof hex from API: {e}"))
            })?,
        });
    }

    let mut account_proofs = Vec::new();
    for request in &builder.ethereum.account {
        if exec_fetcher.network_id() != request.network_id {
            return Err(SdkError::InvalidInput(
                "execution network_id mismatch".into(),
            ));
        }
        let proof = exec_fetcher
            .account(
                request.block_number,
                request.address,
                builder.hashing,
                builder.bankai_block_number,
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

    let mut storage_slot_proofs = Vec::new();
    for request in &builder.ethereum.storage_slot {
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
                    builder.hashing,
                    builder.bankai_block_number,
                )
                .await?,
        );
    }

    Ok(EthereumBatchData {
        block_proof_value,
        execution_header_proofs,
        beacon_header_proofs,
        account_proofs,
        storage_slot_proofs,
        tx_proofs,
        receipt_proofs,
    })
}
