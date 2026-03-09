use std::collections::BTreeMap;

use alloy_rpc_types_eth::{Account as AlloyAccount, Header as ExecutionHeader};
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use bankai_types::api::op_stack::OpStackLightClientProofRequestDto;
use bankai_types::api::proofs::BlockProofPayloadDto;
use bankai_types::inputs::evm::execution::{AccountProof, ReceiptProof, StorageSlotProof, TxProof};
use bankai_types::inputs::evm::op_stack::{OpStackHeaderProof, OpStackMerkleProof};

use super::{get_or_fetch_op_snapshot, op_snapshot_to_witness, parse_fixed_bytes, ProofBatchBuilder};
use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::ApiClient;

pub(super) struct OpStackBatchData {
    pub block_proof_value: Option<BlockProofPayloadDto>,
    pub header_proofs: Vec<OpStackHeaderProof>,
    pub account_proofs: Vec<AccountProof>,
    pub storage_slot_proofs: Vec<StorageSlotProof>,
    pub tx_proofs: Vec<TxProof>,
    pub receipt_proofs: Vec<ReceiptProof>,
}

pub(super) async fn assemble_op_stack_proofs(
    builder: &ProofBatchBuilder<'_>,
    api: &ApiClient,
    filter: &BankaiBlockFilterDto,
) -> SdkResult<OpStackBatchData> {
    let mut block_proof_value = None;
    let mut op_snapshot_by_chain = BTreeMap::new();
    let mut op_header_map: BTreeMap<(String, String), ExecutionHeader> = BTreeMap::new();

    for request in &builder.op_stack.header {
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let snapshot =
            get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone()).await?;
        let header = match (request.block_number, request.header_hash) {
            (Some(block_number), _) => fetcher.header_only(block_number).await?,
            (None, Some(header_hash)) => fetcher.header_only_by_hash(header_hash).await?,
            (None, None) => {
                let header_hash = parse_fixed_bytes(&snapshot.header_hash)?;
                fetcher.header_only_by_hash(header_hash).await?
            }
        };
        op_header_map.insert((request.chain_name.clone(), header.hash.to_string()), header);
    }

    let mut account_proofs = Vec::new();
    for request in &builder.op_stack.account {
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let snapshot =
            get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone()).await?;
        let header = fetcher.header_only(request.block_number).await?;
        op_header_map.insert(
            (request.chain_name.clone(), header.hash.to_string()),
            header.clone(),
        );
        let proof = fetcher.account(request.block_number, request.address).await?;
        account_proofs.push(AccountProof {
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

    let mut storage_slot_proofs = Vec::new();
    for request in &builder.op_stack.storage_slot {
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let _snapshot =
            get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone()).await?;
        let header = fetcher.header_only(request.block_number).await?;
        op_header_map.insert((request.chain_name.clone(), header.hash.to_string()), header);
        storage_slot_proofs.push(
            fetcher
                .storage_slot_proof(request.block_number, request.address, &request.slot_keys)
                .await?,
        );
    }

    let mut tx_proofs = Vec::new();
    for request in &builder.op_stack.tx_proof {
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let snapshot =
            get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone()).await?;
        let proof = fetcher.tx_proof(request.tx_hash).await?;
        if proof.network_id != snapshot.chain_id {
            return Err(SdkError::InvalidInput(format!(
                "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
                request.chain_name, proof.network_id, snapshot.chain_id
            )));
        }
        let header = fetcher.header_only(proof.block_number).await?;
        op_header_map.insert((request.chain_name.clone(), header.hash.to_string()), header);
        tx_proofs.push(proof);
    }

    let mut receipt_proofs = Vec::new();
    for request in &builder.op_stack.receipt_proof {
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let snapshot =
            get_or_fetch_op_snapshot(&mut op_snapshot_by_chain, fetcher, filter.clone()).await?;
        let proof = fetcher.receipt_proof(request.tx_hash).await?;
        if proof.network_id != snapshot.chain_id {
            return Err(SdkError::InvalidInput(format!(
                "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
                request.chain_name, proof.network_id, snapshot.chain_id
            )));
        }
        let header = fetcher.header_only(proof.block_number).await?;
        op_header_map.insert((request.chain_name.clone(), header.hash.to_string()), header);
        receipt_proofs.push(proof);
    }

    let mut header_proofs = Vec::new();
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
            .ok_or_else(|| SdkError::NotFound(format!("missing OP snapshot for {chain_name}")))?;
        let request = OpStackLightClientProofRequestDto {
            filter: filter.clone(),
            hashing_function: builder.hashing,
            header_hashes: header_hashes.clone(),
            proof_format: builder.proof_format,
        };
        let proof = api.op_stack().light_client_proof(&chain_name, &request).await?;
        if block_proof_value.is_none() {
            block_proof_value = Some(proof.block_proof.proof.clone());
        }

        let mut mmr_by_hash = BTreeMap::new();
        for mmr_proof in proof.mmr_proofs {
            mmr_by_hash.insert(mmr_proof.header_hash.clone(), mmr_proof);
        }

        let merkle_proof: OpStackMerkleProof = proof.merkle_proof.try_into().map_err(|e| {
            SdkError::InvalidInput(format!("invalid OP merkle proof hex from API: {e}"))
        })?;

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
            header_proofs.push(OpStackHeaderProof {
                header: header.clone(),
                snapshot: op_snapshot_to_witness(snapshot.clone())?,
                merkle_proof: merkle_proof.clone(),
                mmr_proof: mmr_proof.clone().try_into().map_err(|e| {
                    SdkError::InvalidInput(format!("invalid OP MMR proof hex from API: {e}"))
                })?,
            });
        }
    }

    Ok(OpStackBatchData {
        block_proof_value,
        header_proofs,
        account_proofs,
        storage_slot_proofs,
        tx_proofs,
        receipt_proofs,
    })
}
