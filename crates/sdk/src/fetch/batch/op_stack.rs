use std::collections::BTreeMap;
use std::time::Instant;

use alloy_rpc_types_eth::{Account as AlloyAccount, Header as ExecutionHeader};
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use bankai_types::api::op_stack::OpStackLightClientProofRequestDto;
use bankai_types::api::proofs::BankaiBlockProofDto;
use bankai_types::inputs::evm::execution::{AccountProof, ReceiptProof, StorageSlotProof, TxProof};
use bankai_types::inputs::evm::op_stack::{OpStackHeaderProof, OpStackMerkleProof};

use super::{validate_bankai_block_proof, ProofBatchBuilder};
use crate::debug;
use crate::errors::{SdkError, SdkResult};
use crate::fetch::api::ApiClient;

pub(super) struct OpStackBatchData {
    pub block_proof: Option<BankaiBlockProofDto>,
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
    debug::log(format!(
        "assembling op-stack proofs headers={} accounts={} storage_slots={} txs={} receipts={}",
        builder.op_stack.header.len(),
        builder.op_stack.account.len(),
        builder.op_stack.storage_slot.len(),
        builder.op_stack.tx_proof.len(),
        builder.op_stack.receipt_proof.len(),
    ));

    let mut block_proof = None;
    let mut op_header_map: BTreeMap<(String, String), ExecutionHeader> = BTreeMap::new();
    let mut op_chain_ids = BTreeMap::new();
    let mut op_committed_heights = BTreeMap::new();

    for request in &builder.op_stack.header {
        let request_start = Instant::now();
        debug::log(format!(
            "op-stack header request chain={} block={:?} header_hash_present={}",
            request.chain_name,
            request.block_number,
            request.header_hash.is_some()
        ));
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let header = match (request.block_number, request.header_hash) {
            (Some(block_number), _) => fetcher.header_only(block_number).await?,
            (None, Some(header_hash)) => fetcher.header_only_by_hash(header_hash).await?,
            (None, None) => {
                let block_number = get_or_fetch_op_committed_height(
                    &mut op_committed_heights,
                    api,
                    &request.chain_name,
                    filter,
                )
                .await?;
                fetcher.header_only(block_number).await?
            }
        };
        op_header_map.insert(
            (request.chain_name.clone(), header.hash.to_string()),
            header,
        );
        debug::log(format!(
            "op-stack header request chain={} completed in {} ms",
            request.chain_name,
            debug::elapsed_ms(request_start)
        ));
    }

    let mut account_proofs = Vec::new();
    for request in &builder.op_stack.account {
        let request_start = Instant::now();
        debug::log(format!(
            "op-stack account request chain={} block={} address={}",
            request.chain_name, request.block_number, request.address
        ));
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let chain_id = get_or_fetch_op_chain_id(&mut op_chain_ids, fetcher).await?;
        let header = fetcher.header_only(request.block_number).await?;
        op_header_map.insert(
            (request.chain_name.clone(), header.hash.to_string()),
            header.clone(),
        );
        let proof = fetcher
            .account(request.block_number, request.address)
            .await?;
        account_proofs.push(AccountProof {
            account: AlloyAccount {
                balance: proof.balance,
                nonce: proof.nonce,
                code_hash: proof.code_hash,
                storage_root: proof.storage_hash,
            },
            address: request.address,
            network_id: chain_id,
            block_number: request.block_number,
            state_root: header.state_root,
            mpt_proof: proof.account_proof,
        });
        debug::log(format!(
            "op-stack account request chain={} block={} completed in {} ms",
            request.chain_name,
            request.block_number,
            debug::elapsed_ms(request_start)
        ));
    }

    let mut storage_slot_proofs = Vec::new();
    for request in &builder.op_stack.storage_slot {
        let request_start = Instant::now();
        debug::log(format!(
            "op-stack storage request chain={} block={} slots={}",
            request.chain_name,
            request.block_number,
            request.slot_keys.len()
        ));
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let header = fetcher.header_only(request.block_number).await?;
        op_header_map.insert(
            (request.chain_name.clone(), header.hash.to_string()),
            header,
        );
        storage_slot_proofs.push(
            fetcher
                .storage_slot_proof(request.block_number, request.address, &request.slot_keys)
                .await?,
        );
        debug::log(format!(
            "op-stack storage request chain={} block={} completed in {} ms",
            request.chain_name,
            request.block_number,
            debug::elapsed_ms(request_start)
        ));
    }

    let mut tx_proofs = Vec::new();
    for request in &builder.op_stack.tx_proof {
        let request_start = Instant::now();
        debug::log(format!(
            "op-stack tx request chain={} tx_hash={}",
            request.chain_name, request.tx_hash
        ));
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let chain_id = get_or_fetch_op_chain_id(&mut op_chain_ids, fetcher).await?;
        let proof = fetcher.tx_proof(request.tx_hash).await?;
        if proof.network_id != chain_id {
            return Err(SdkError::InvalidInput(format!(
                "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
                request.chain_name, proof.network_id, chain_id
            )));
        }
        let header = fetcher.header_only(proof.block_number).await?;
        op_header_map.insert(
            (request.chain_name.clone(), header.hash.to_string()),
            header,
        );
        tx_proofs.push(proof);
        debug::log(format!(
            "op-stack tx request chain={} completed in {} ms",
            request.chain_name,
            debug::elapsed_ms(request_start)
        ));
    }

    let mut receipt_proofs = Vec::new();
    for request in &builder.op_stack.receipt_proof {
        let request_start = Instant::now();
        debug::log(format!(
            "op-stack receipt request chain={} tx_hash={}",
            request.chain_name, request.tx_hash
        ));
        let fetcher = builder.bankai.op_stack(&request.chain_name)?;
        let chain_id = get_or_fetch_op_chain_id(&mut op_chain_ids, fetcher).await?;
        let proof = fetcher.receipt_proof(request.tx_hash).await?;
        if proof.network_id != chain_id {
            return Err(SdkError::InvalidInput(format!(
                "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
                request.chain_name, proof.network_id, chain_id
            )));
        }
        let header = fetcher.header_only(proof.block_number).await?;
        op_header_map.insert(
            (request.chain_name.clone(), header.hash.to_string()),
            header,
        );
        receipt_proofs.push(proof);
        debug::log(format!(
            "op-stack receipt request chain={} completed in {} ms",
            request.chain_name,
            debug::elapsed_ms(request_start)
        ));
    }

    let mut header_proofs = Vec::new();
    let mut op_header_hashes_by_chain: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (chain_name, header_hash) in op_header_map.keys() {
        op_header_hashes_by_chain
            .entry(chain_name.clone())
            .or_default()
            .push(header_hash.clone());
    }

    for (chain_name, header_hashes) in op_header_hashes_by_chain {
        let request = OpStackLightClientProofRequestDto {
            filter: filter.clone(),
            hashing_function: builder.hashing,
            header_hashes: header_hashes.clone(),
            proof_format: builder.proof_format,
        };
        let request_start = Instant::now();
        let proof_result = api
            .op_stack()
            .light_client_proof(&chain_name, &request)
            .await;
        debug::log_result(
            format!(
                "api op-stack light_client_proof chain={} headers={}",
                chain_name,
                header_hashes.len()
            ),
            request_start,
            &proof_result,
        );
        let proof = proof_result?;
        validate_bankai_block_proof(&proof.block_proof, builder.bankai_block_number)?;
        if proof.merkle_proof.bankai_block_number != builder.bankai_block_number {
            return Err(SdkError::InvalidInput(format!(
                "OP merkle proof bankai block mismatch for {}: expected {}, got {}",
                chain_name, builder.bankai_block_number, proof.merkle_proof.bankai_block_number
            )));
        }
        if block_proof.is_none() {
            block_proof = Some(proof.block_proof.clone());
        }

        let snapshot = proof.snapshot.clone();
        let expected_chain_id =
            get_or_fetch_op_chain_id(&mut op_chain_ids, builder.bankai.op_stack(&chain_name)?)
                .await?;
        if snapshot.chain_id != expected_chain_id {
            return Err(SdkError::InvalidInput(format!(
                "OP snapshot chain_id mismatch for {}: rpc returned {}, proof returned {}",
                chain_name, expected_chain_id, snapshot.chain_id
            )));
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
                        "missing OP header for chain {chain_name} and hash {header_hash}"
                    ))
                })?;
            let mmr_proof = mmr_by_hash.get(&header_hash).ok_or_else(|| {
                SdkError::NotFound(format!(
                    "missing OP MMR proof for chain {chain_name} and hash {header_hash}"
                ))
            })?;
            header_proofs.push(OpStackHeaderProof {
                header: header.clone(),
                snapshot: snapshot.clone(),
                merkle_proof: merkle_proof.clone(),
                mmr_proof: mmr_proof.clone().try_into().map_err(|e| {
                    SdkError::InvalidInput(format!("invalid OP MMR proof hex from API: {e}"))
                })?,
            });
        }
    }

    Ok(OpStackBatchData {
        block_proof,
        header_proofs,
        account_proofs,
        storage_slot_proofs,
        tx_proofs,
        receipt_proofs,
    })
}

async fn get_or_fetch_op_chain_id(
    chain_ids: &mut BTreeMap<String, u64>,
    fetcher: &crate::fetch::evm::op_stack::OpStackChainFetcher,
) -> SdkResult<u64> {
    if let Some(chain_id) = chain_ids.get(fetcher.chain_name()) {
        return Ok(*chain_id);
    }

    let chain_id = fetcher.chain_id().await?;
    chain_ids.insert(fetcher.chain_name().to_string(), chain_id);
    Ok(chain_id)
}

async fn get_or_fetch_op_committed_height(
    committed_heights: &mut BTreeMap<String, u64>,
    api: &ApiClient,
    chain_name: &str,
    filter: &BankaiBlockFilterDto,
) -> SdkResult<u64> {
    if let Some(height) = committed_heights.get(chain_name) {
        return Ok(*height);
    }

    let height = api.op_stack().height(chain_name, filter).await?.height;
    committed_heights.insert(chain_name.to_string(), height);
    Ok(height)
}
