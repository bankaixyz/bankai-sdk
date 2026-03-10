use alloy_primitives::{hex::FromHex, FixedBytes};
use anyhow::{anyhow, Context, Result};
use bankai_sdk::parse_block_proof_payload;
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use bankai_types::api::op_stack::{
    OpMerkleProofDto, OpStackLightClientProofDto, OpStackLightClientProofRequestDto,
    OpStackMerkleProofRequestDto, OpStackMmrProofRequestDto,
};
use bankai_types::api::proofs::BankaiBlockProofDto;
use bankai_types::api::proofs::EthereumLightClientProofDto;
use bankai_types::block::OpChainClient;
use bankai_types::common::{HashingFunction, ProofFormat};
use bankai_types::inputs::evm::op_stack::OpStackMerkleProof;
use bankai_types::inputs::evm::MmrProof;
use bankai_verify::bankai::mmr::MmrVerifier;
use bankai_verify::bankai::stwo::verify_stwo_proof;
use bankai_verify::evm::op_stack::OpStackVerifier;

use crate::compat::case::{
    BankaiMmrProofSource, LightClientProofSource, MatrixScope, MerkleProofSource, MmrProofSource,
    ProofHashSource,
};
use crate::compat::context::CompatContext;

use super::{
    api_mmr_dto_to_mmr, assert_bankai_mmr_proofs_equal, format_variant, hex_eq,
    validate_bankai_mmr_contract, validate_mmr_proof_contract,
};

pub(super) async fn run_proof_hash_consistency(
    ctx: &CompatContext,
    source: ProofHashSource,
    scope: MatrixScope,
) -> Result<()> {
    match source {
        ProofHashSource::BlocksBlockProof => {
            let filters = match scope {
                MatrixScope::Core => ctx.filter_cases_core().await?,
                MatrixScope::Edge => ctx.filter_cases_edge().await?,
            };
            let hashings = match scope {
                MatrixScope::Core => ctx.hashing_cases_core(),
                MatrixScope::Edge => ctx.hashing_cases_edge(),
            };
            let proof_formats = ctx.proof_format_cases_core();

            for filter_case in filters {
                let targets = match scope {
                    MatrixScope::Core => {
                        ctx.target_block_cases_core_for_filter(&filter_case.filter)
                            .await?
                    }
                    MatrixScope::Edge => {
                        ctx.target_block_cases_edge_for_filter(&filter_case.filter)
                            .await?
                    }
                };
                for target_case in &targets {
                    for hashing_case in &hashings {
                        for proof_case in &proof_formats {
                            let variant = format_variant(
                                &format!(
                                    "{}, {}, {}, {}",
                                    filter_case.label,
                                    target_case.label,
                                    hashing_case.label,
                                    proof_case.label
                                ),
                                scope,
                            );

                            let pinned_filter =
                                pinned_filter_for_consistency(ctx, &filter_case.filter, scope)
                                    .await?;

                            let request = ctx.bankai_block_proof_request_for(
                                pinned_filter.clone(),
                                target_case.target_block.clone(),
                                hashing_case.hashing_function,
                                proof_case.proof_format,
                            );

                            let response = ctx
                                .sdk
                                .api
                                .blocks()
                                .block_proof(&request)
                                .await
                                .with_context(|| {
                                    format!(
                                        "blocks block_proof failed for proof consistency {variant}"
                                    )
                                })?;

                            let block_proof = response.block_proof;
                            assert_block_proof_payload_matches(
                                &block_proof.proof,
                                proof_case.proof_format,
                                &variant,
                            )?;
                            let stwo_proof = parse_block_proof_payload(block_proof.proof)
                                .with_context(|| {
                                    format!("failed to parse block proof payload for {variant}")
                                })?;
                            let _ = verify_stwo_proof(stwo_proof).with_context(|| {
                                format!("STWO proof hash-output verification failed for {variant}")
                            })?;

                            let standalone_mmr_request = ctx.bankai_mmr_request_for(
                                request.filter.clone(),
                                request.target_block.clone(),
                                request.hashing_function,
                            );
                            let standalone_mmr = ctx
                                .sdk
                                .api
                                .blocks()
                                .mmr_proof(&standalone_mmr_request)
                                .await
                                .with_context(|| {
                                    format!(
                                        "blocks mmr_proof failed while checking block_proof consistency ({variant})"
                                    )
                                })?;

                            validate_bankai_mmr_contract(
                                &response.mmr_proof,
                                &standalone_mmr_request,
                            )
                            .with_context(|| {
                                format!(
                                    "block_proof returned invalid mmr_proof contract for {variant}"
                                )
                            })?;
                            assert_bankai_mmr_proofs_equal(&response.mmr_proof, &standalone_mmr)
                                .with_context(|| {
                                    format!(
                                        "block_proof mmr_proof mismatch with standalone /v1/blocks/mmr_proof for {variant}"
                                    )
                                })?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn pinned_filter_for_consistency(
    ctx: &CompatContext,
    filter: &BankaiBlockFilterDto,
    scope: MatrixScope,
) -> Result<BankaiBlockFilterDto> {
    match scope {
        MatrixScope::Core => {
            let reference_block = ctx.resolved_reference_block_for_filter(filter).await?;
            Ok(BankaiBlockFilterDto::with_bankai_block_number(
                reference_block,
            ))
        }
        MatrixScope::Edge => Ok(filter.clone()),
    }
}

pub(super) async fn run_merkle_proof_verify(
    ctx: &CompatContext,
    source: MerkleProofSource,
    scope: MatrixScope,
) -> Result<()> {
    match source {
        MerkleProofSource::OpStackFromSnapshot => {
            let filters = match scope {
                MatrixScope::Core => ctx.filter_cases_core().await?,
                MatrixScope::Edge => ctx.filter_cases_edge().await?,
            };

            let name = ctx.op_chain_name().await?;

            for filter_case in filters {
                let variant = format_variant(&filter_case.label, scope);
                let request = OpStackMerkleProofRequestDto {
                    filter: filter_case.filter.clone(),
                };
                let proof = ctx
                    .sdk
                    .api
                    .op_stack()
                    .merkle_proof(&name, &request)
                    .await
                    .with_context(|| {
                        format!("op_stack merkle_proof failed for chain={name}, {variant}")
                    })?;

                let snapshot = ctx
                    .sdk
                    .api
                    .op_stack()
                    .snapshot(&name, &filter_case.filter)
                    .await
                    .with_context(|| {
                        format!(
                            "op_stack snapshot failed while building merkle verification request for chain={name}, {variant}"
                        )
                    })?;
                let op_chains_root = trusted_op_chains_root_for_block(
                    ctx,
                    proof.bankai_block_number,
                    None,
                    &variant,
                )
                .await?;
                let trusted_snapshot = op_snapshot_summary_to_client(&snapshot, &name, &variant)?;
                let merkle_proof = op_merkle_dto_to_input(&proof)
                    .with_context(|| format!("invalid OP merkle proof payload for {variant}"))?;

                if trusted_snapshot.commitment_leaf_hash() != merkle_proof.leaf_hash {
                    return Err(anyhow!(
                        "op_stack merkle leaf mismatch for chain={name}, {variant}: expected {}, got {}",
                        trusted_snapshot.commitment_leaf_hash(),
                        merkle_proof.leaf_hash
                    ));
                }

                OpStackVerifier::verify_merkle_proof(&merkle_proof, op_chains_root).with_context(
                    || format!("op_stack merkle verification failed for chain={name}, {variant}"),
                )?;
            }
        }
    }

    Ok(())
}

pub(super) async fn run_mmr_verify(
    ctx: &CompatContext,
    source: MmrProofSource,
    scope: MatrixScope,
) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };
    let hashings = match scope {
        MatrixScope::Core => ctx.hashing_cases_core(),
        MatrixScope::Edge => ctx.hashing_cases_edge(),
    };

    for filter_case in filters {
        for hashing_case in &hashings {
            let variant = format_variant(
                &format!("{}, {}", filter_case.label, hashing_case.label),
                scope,
            );

            let mmr: MmrProof = match source {
                MmrProofSource::EthereumBeaconFromSnapshot => {
                    let request = match scope {
                        MatrixScope::Core => {
                            ctx.beacon_mmr_proof_request_for(
                                &filter_case.filter,
                                hashing_case.hashing_function,
                            )
                            .await
                        }
                        MatrixScope::Edge => {
                            let mut req = ctx.beacon_mmr_proof_request().await?;
                            req.filter = filter_case.filter.clone();
                            req.hashing_function = hashing_case.hashing_function;
                            Ok(req)
                        }
                    }
                    .with_context(|| format!("failed building beacon mmr request for {variant}"))?;

                    let proof = ctx
                        .sdk
                        .api
                        .ethereum()
                        .beacon()
                        .mmr_proof(&request)
                        .await
                        .with_context(|| format!("beacon mmr_proof failed for {variant}"))?;
                    validate_mmr_proof_contract(&proof, &request.header_hash)
                        .with_context(|| format!("beacon mmr contract invalid for {variant}"))?;
                    if proof.hashing_function != request.hashing_function {
                        return Err(anyhow!(
                            "beacon mmr hashing_function mismatch for {variant}: expected {:?}, got {:?}",
                            request.hashing_function,
                            proof.hashing_function
                        ));
                    }
                    api_mmr_dto_to_mmr(&proof).with_context(|| {
                        format!("failed converting beacon mmr proof for {variant}")
                    })?
                }
                MmrProofSource::EthereumExecutionFromSnapshot => {
                    let request = match scope {
                        MatrixScope::Core => {
                            ctx.execution_mmr_proof_request_for(
                                &filter_case.filter,
                                hashing_case.hashing_function,
                            )
                            .await
                        }
                        MatrixScope::Edge => {
                            let mut req = ctx.execution_mmr_proof_request().await?;
                            req.filter = filter_case.filter.clone();
                            req.hashing_function = hashing_case.hashing_function;
                            Ok(req)
                        }
                    }
                    .with_context(|| {
                        format!("failed building execution mmr request for {variant}")
                    })?;

                    let proof = ctx
                        .sdk
                        .api
                        .ethereum()
                        .execution()
                        .mmr_proof(&request)
                        .await
                        .with_context(|| format!("execution mmr_proof failed for {variant}"))?;
                    validate_mmr_proof_contract(&proof, &request.header_hash)
                        .with_context(|| format!("execution mmr contract invalid for {variant}"))?;
                    if proof.hashing_function != request.hashing_function {
                        return Err(anyhow!(
                            "execution mmr hashing_function mismatch for {variant}: expected {:?}, got {:?}",
                            request.hashing_function,
                            proof.hashing_function
                        ));
                    }
                    api_mmr_dto_to_mmr(&proof).with_context(|| {
                        format!("failed converting execution mmr proof for {variant}")
                    })?
                }
                MmrProofSource::OpStackFromSnapshot => {
                    let name = ctx.op_chain_name().await?;
                    let snapshot = ctx
                        .sdk
                        .api
                        .op_stack()
                        .snapshot(&name, &filter_case.filter)
                        .await
                        .with_context(|| {
                            format!(
                                "op_stack snapshot failed while building mmr verification request for chain={name}, {variant}"
                            )
                        })?;
                    let request = OpStackMmrProofRequestDto {
                        filter: filter_case.filter.clone(),
                        hashing_function: hashing_case.hashing_function,
                        header_hash: snapshot.header_hash.clone(),
                    };
                    let proof = ctx
                        .sdk
                        .api
                        .op_stack()
                        .mmr_proof(&name, &request)
                        .await
                        .with_context(|| {
                            format!("op_stack mmr_proof failed for chain={name}, {variant}")
                        })?;

                    validate_mmr_proof_contract(&proof.mmr_proof, &request.header_hash)
                        .with_context(|| {
                            format!("op_stack mmr contract invalid for chain={name}, {variant}")
                        })?;
                    if proof.mmr_proof.hashing_function != request.hashing_function {
                        return Err(anyhow!(
                            "op_stack mmr hashing_function mismatch for chain={name}, {variant}: expected {:?}, got {:?}",
                            request.hashing_function,
                            proof.mmr_proof.hashing_function
                        ));
                    }

                    let op_chains_root = trusted_op_chains_root_for_block(
                        ctx,
                        proof.merkle_proof.bankai_block_number,
                        None,
                        &variant,
                    )
                    .await?;
                    let trusted_snapshot =
                        op_snapshot_summary_to_client(&snapshot, &name, &variant)?;
                    let merkle_proof = op_merkle_dto_to_input(&proof.merkle_proof).with_context(
                        || {
                            format!(
                                "failed converting op_stack merkle proof for chain={name}, {variant}"
                            )
                        },
                    )?;

                    if trusted_snapshot.commitment_leaf_hash() != merkle_proof.leaf_hash {
                        return Err(anyhow!(
                            "op_stack merkle leaf mismatch for chain={name}, {variant}: expected {}, got {}",
                            trusted_snapshot.commitment_leaf_hash(),
                            merkle_proof.leaf_hash
                        ));
                    }

                    OpStackVerifier::verify_merkle_proof(&merkle_proof, op_chains_root)
                        .with_context(|| {
                            format!(
                                "op_stack merkle verification failed for chain={name}, {variant}"
                            )
                        })?;

                    let expected_snapshot_root =
                        snapshot_mmr_root(&trusted_snapshot, request.hashing_function);
                    if proof.mmr_proof.root != expected_snapshot_root.to_string() {
                        return Err(anyhow!(
                            "op_stack mmr root mismatch for chain={name}, {variant}: expected {}, got {}",
                            expected_snapshot_root,
                            proof.mmr_proof.root
                        ));
                    }

                    api_mmr_dto_to_mmr(&proof.mmr_proof).with_context(|| {
                        format!("failed converting op_stack mmr proof for chain={name}, {variant}")
                    })?
                }
            };

            MmrVerifier::verify_mmr_proof(&mmr)
                .with_context(|| format!("MMR proof verification failed for {variant}"))?;
        }
    }

    Ok(())
}

pub(super) async fn run_bankai_mmr_verify(
    ctx: &CompatContext,
    source: BankaiMmrProofSource,
    scope: MatrixScope,
) -> Result<()> {
    match source {
        BankaiMmrProofSource::BlocksMmrProofEndpoint => {
            let filters = match scope {
                MatrixScope::Core => ctx.filter_cases_core().await?,
                MatrixScope::Edge => ctx.filter_cases_edge().await?,
            };
            let hashings = match scope {
                MatrixScope::Core => ctx.hashing_cases_core(),
                MatrixScope::Edge => ctx.hashing_cases_edge(),
            };

            for filter_case in filters {
                let targets = match scope {
                    MatrixScope::Core => {
                        ctx.target_block_cases_core_for_filter(&filter_case.filter)
                            .await?
                    }
                    MatrixScope::Edge => {
                        ctx.target_block_cases_edge_for_filter(&filter_case.filter)
                            .await?
                    }
                };
                for target_case in &targets {
                    for hashing_case in &hashings {
                        let variant = format_variant(
                            &format!(
                                "{}, {}, {}",
                                filter_case.label, target_case.label, hashing_case.label
                            ),
                            scope,
                        );

                        let request = ctx.bankai_mmr_request_for(
                            filter_case.filter.clone(),
                            target_case.target_block.clone(),
                            hashing_case.hashing_function,
                        );

                        let mmr_proof = ctx
                            .sdk
                            .api
                            .blocks()
                            .mmr_proof(&request)
                            .await
                            .with_context(|| format!("blocks mmr_proof failed for {variant}"))?;
                        validate_bankai_mmr_contract(&mmr_proof, &request).with_context(|| {
                            format!("bankai mmr proof contract validation failed for {variant}")
                        })?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub(super) async fn run_light_client_proof_verify(
    ctx: &CompatContext,
    source: LightClientProofSource,
    scope: MatrixScope,
) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };
    let hashings = match scope {
        MatrixScope::Core => ctx.hashing_cases_core(),
        MatrixScope::Edge => ctx.hashing_cases_edge(),
    };
    let proof_formats = ctx.proof_format_cases_core();

    for filter_case in filters {
        for hashing_case in &hashings {
            for proof_case in &proof_formats {
                let variant = format_variant(
                    &format!(
                        "{}, {}, {}",
                        filter_case.label, hashing_case.label, proof_case.label
                    ),
                    scope,
                );

                match source {
                    LightClientProofSource::EthereumBeaconFromSnapshot => {
                        let request = match scope {
                            MatrixScope::Core => {
                                ctx.beacon_light_client_request_for(
                                    &filter_case.filter,
                                    hashing_case.hashing_function,
                                    proof_case.proof_format,
                                )
                                .await
                            }
                            MatrixScope::Edge => {
                                let mut req = ctx.beacon_light_client_request().await?;
                                req.filter = filter_case.filter.clone();
                                req.hashing_function = hashing_case.hashing_function;
                                req.proof_format = proof_case.proof_format;
                                Ok(req)
                            }
                        }
                        .with_context(|| {
                            format!("failed building beacon light client request for {variant}")
                        })?;

                        let expected_root = ctx
                            .sdk
                            .api
                            .ethereum()
                            .beacon()
                            .mmr_root(&request.filter)
                            .await
                            .with_context(|| {
                                format!(
                                    "beacon mmr_root failed during light client verification for {variant}"
                                )
                            })?
                            .keccak_root;
                        let proof = ctx
                            .sdk
                            .api
                            .ethereum()
                            .beacon()
                            .light_client_proof(&request)
                            .await
                            .with_context(|| {
                                format!("beacon light_client_proof failed for {variant}")
                            })?;
                        verify_light_client_bundle(
                            &proof,
                            &request.header_hashes,
                            &expected_root,
                            request.hashing_function,
                            &variant,
                        )?;
                    }
                    LightClientProofSource::EthereumExecutionFromSnapshot => {
                        let request = match scope {
                            MatrixScope::Core => {
                                ctx.execution_light_client_request_for(
                                    &filter_case.filter,
                                    hashing_case.hashing_function,
                                    proof_case.proof_format,
                                )
                                .await
                            }
                            MatrixScope::Edge => {
                                let mut req = ctx.execution_light_client_request().await?;
                                req.filter = filter_case.filter.clone();
                                req.hashing_function = hashing_case.hashing_function;
                                req.proof_format = proof_case.proof_format;
                                Ok(req)
                            }
                        }
                        .with_context(|| {
                            format!("failed building execution light client request for {variant}")
                        })?;

                        let expected_root = ctx
                            .sdk
                            .api
                            .ethereum()
                            .execution()
                            .mmr_root(&request.filter)
                            .await
                            .with_context(|| {
                                format!(
                                    "execution mmr_root failed during light client verification for {variant}"
                                )
                            })?
                            .keccak_root;
                        let proof = ctx
                            .sdk
                            .api
                            .ethereum()
                            .execution()
                            .light_client_proof(&request)
                            .await
                            .with_context(|| {
                                format!("execution light_client_proof failed for {variant}")
                            })?;
                        verify_light_client_bundle(
                            &proof,
                            &request.header_hashes,
                            &expected_root,
                            request.hashing_function,
                            &variant,
                        )?;
                    }
                    LightClientProofSource::OpStackFromSnapshot => {
                        let name = ctx.op_chain_name().await?;
                        let snapshot = ctx
                            .sdk
                            .api
                            .op_stack()
                            .snapshot(&name, &filter_case.filter)
                            .await
                            .with_context(|| {
                                format!(
                                    "op_stack snapshot failed while building light client request for chain={name}, {variant}"
                                )
                            })?;
                        let request = OpStackLightClientProofRequestDto {
                            filter: filter_case.filter.clone(),
                            hashing_function: hashing_case.hashing_function,
                            header_hashes: vec![snapshot.header_hash.clone()],
                            proof_format: proof_case.proof_format,
                        };
                        let proof = ctx
                            .sdk
                            .api
                            .op_stack()
                            .light_client_proof(&name, &request)
                            .await
                            .with_context(|| {
                                format!(
                                    "op_stack light_client_proof failed for chain={name}, {variant}"
                                )
                            })?;

                        verify_op_stack_light_client_bundle(
                            &name,
                            &proof,
                            &request.header_hashes,
                            proof.block_proof.block_number,
                            request.hashing_function,
                            &variant,
                        )
                        .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn verify_light_client_bundle(
    proof: &EthereumLightClientProofDto,
    requested_header_hashes: &[String],
    expected_mmr_root: &str,
    expected_hashing_function: HashingFunction,
    variant: &str,
) -> Result<()> {
    verify_bankai_block_proof(
        &proof.block_proof,
        Some(proof.block_proof.block_number),
        variant,
    )?;

    if proof.mmr_proofs.is_empty() {
        return Err(anyhow!(
            "light_client_proof returned no mmr_proofs for {variant}"
        ));
    }

    for mmr_dto in &proof.mmr_proofs {
        if !requested_header_hashes
            .iter()
            .any(|header| hex_eq(header, &mmr_dto.header_hash))
        {
            return Err(anyhow!(
                "light_client_proof returned unexpected header hash {} for {variant}",
                mmr_dto.header_hash
            ));
        }

        if !hex_eq(expected_mmr_root, &mmr_dto.root) {
            return Err(anyhow!(
                "light_client_proof root mismatch for {variant}: expected {}, got {}",
                expected_mmr_root,
                mmr_dto.root
            ));
        }
        if mmr_dto.hashing_function != expected_hashing_function {
            return Err(anyhow!(
                "light_client_proof hashing_function mismatch for {variant}: expected {:?}, got {:?}",
                expected_hashing_function,
                mmr_dto.hashing_function
            ));
        }

        let mmr = api_mmr_dto_to_mmr(mmr_dto)
            .with_context(|| format!("failed converting light client MMR proof for {variant}"))?;
        MmrVerifier::verify_mmr_proof(&mmr)
            .with_context(|| format!("light client mmr proof verification failed for {variant}"))?;
    }

    Ok(())
}

async fn verify_op_stack_light_client_bundle(
    name: &str,
    proof: &OpStackLightClientProofDto,
    requested_header_hashes: &[String],
    expected_block_number: u64,
    expected_hashing_function: HashingFunction,
    variant: &str,
) -> Result<()> {
    if proof.merkle_proof.bankai_block_number != proof.block_proof.block_number {
        return Err(anyhow!(
            "OP light_client_proof bankai block mismatch for {variant}: merkle_proof={}, block_proof={}",
            proof.merkle_proof.bankai_block_number,
            proof.block_proof.block_number
        ));
    }
    verify_bankai_block_proof(&proof.block_proof, Some(expected_block_number), variant)?;

    if proof.mmr_proofs.len() != requested_header_hashes.len() {
        return Err(anyhow!(
            "OP light_client_proof returned {} MMR proofs for {variant}; expected {}",
            proof.mmr_proofs.len(),
            requested_header_hashes.len()
        ));
    }

    let trusted_snapshot = proof.snapshot.clone();
    if trusted_snapshot.chain_id != proof.merkle_proof.chain_id {
        return Err(anyhow!(
            "OP light_client_proof chain_id mismatch for chain={name}, {variant}: snapshot={}, merkle={}",
            trusted_snapshot.chain_id,
            proof.merkle_proof.chain_id
        ));
    }

    let merkle_proof = op_merkle_dto_to_input(&proof.merkle_proof)
        .with_context(|| format!("invalid OP merkle proof payload for {variant}"))?;
    if trusted_snapshot.commitment_leaf_hash() != merkle_proof.leaf_hash {
        return Err(anyhow!(
            "OP light_client_proof merkle leaf mismatch for chain={name}, {variant}: expected {}, got {}",
            trusted_snapshot.commitment_leaf_hash(),
            merkle_proof.leaf_hash
        ));
    }
    if merkle_proof.root != proof.block_proof.block.block.op_chains.root {
        return Err(anyhow!(
            "OP light_client_proof merkle root mismatch for chain={name}, {variant}: expected {}, got {}",
            proof.block_proof.block.block.op_chains.root,
            merkle_proof.root
        ));
    }

    for mmr_dto in &proof.mmr_proofs {
        if !requested_header_hashes
            .iter()
            .any(|header| hex_eq(header, &mmr_dto.header_hash))
        {
            return Err(anyhow!(
                "OP light_client_proof returned unexpected header hash {} for {variant}",
                mmr_dto.header_hash
            ));
        }
        if mmr_dto.hashing_function != expected_hashing_function {
            return Err(anyhow!(
                "OP light_client_proof hashing_function mismatch for {variant}: expected {:?}, got {:?}",
                expected_hashing_function,
                mmr_dto.hashing_function
            ));
        }

        let expected_snapshot_root =
            snapshot_mmr_root(&trusted_snapshot, expected_hashing_function);
        if !hex_eq(&mmr_dto.root, &expected_snapshot_root.to_string()) {
            return Err(anyhow!(
                "OP light_client_proof root mismatch for {variant}: expected {}, got {}",
                expected_snapshot_root,
                mmr_dto.root
            ));
        }

        let mmr = api_mmr_dto_to_mmr(mmr_dto).with_context(|| {
            format!("failed converting OP light client MMR proof for {variant}")
        })?;
        MmrVerifier::verify_mmr_proof(&mmr)
            .with_context(|| format!("OP light client mmr verification failed for {variant}"))?;
    }

    Ok(())
}

fn verify_bankai_block_proof(
    block_proof: &BankaiBlockProofDto,
    expected_block_number: Option<u64>,
    variant: &str,
) -> Result<()> {
    if block_proof.block.block.block_number != block_proof.block_number {
        return Err(anyhow!(
            "block witness mismatch for {variant}: proof={}, block={}",
            block_proof.block_number,
            block_proof.block.block.block_number
        ));
    }
    if let Some(expected) = expected_block_number {
        if block_proof.block_number != expected {
            return Err(anyhow!(
                "block proof number mismatch for {variant}: expected {}, got {}",
                expected,
                block_proof.block_number
            ));
        }
    }

    let stwo_proof = parse_block_proof_payload(block_proof.proof.clone())
        .with_context(|| format!("failed to parse block proof payload for {variant}"))?;
    let hash_output = verify_stwo_proof(stwo_proof)
        .with_context(|| format!("STWO proof hash-output verification failed for {variant}"))?;
    let expected_block_hash = block_proof.block.block.compute_block_hash_keccak();
    if hash_output.block_hash != expected_block_hash {
        return Err(anyhow!(
            "block hash mismatch for {variant}: expected {}, got {}",
            expected_block_hash,
            hash_output.block_hash
        ));
    }

    Ok(())
}

async fn trusted_op_chains_root_for_block(
    ctx: &CompatContext,
    block_number: u64,
    block_proof: Option<&BankaiBlockProofDto>,
    variant: &str,
) -> Result<FixedBytes<32>> {
    let block_proof = match block_proof {
        Some(block_proof) => block_proof.clone(),
        None => ctx
            .sdk
            .api
            .blocks()
            .proof(block_number)
            .await
            .with_context(|| format!("failed to fetch block proof {block_number} for {variant}"))?,
    };

    verify_bankai_block_proof(&block_proof, Some(block_number), variant)?;
    Ok(block_proof.block.block.op_chains.root)
}

fn op_snapshot_summary_to_client(
    snapshot: &bankai_types::api::op_stack::OpChainSnapshotSummaryDto,
    name: &str,
    variant: &str,
) -> Result<bankai_types::block::OpChainClient> {
    Ok(bankai_types::block::OpChainClient {
        chain_id: snapshot.chain_id,
        block_number: snapshot.end_height,
        header_hash: FixedBytes::from_hex(&snapshot.header_hash).map_err(|_| {
            anyhow!(
                "invalid OP snapshot header_hash for chain={name}, {variant}: {}",
                snapshot.header_hash
            )
        })?,
        l1_submission_block: snapshot.l1_submission_block,
        mmr_root_keccak: FixedBytes::from_hex(&snapshot.mmr_roots.keccak_root).map_err(|_| {
            anyhow!(
                "invalid OP snapshot keccak root for chain={name}, {variant}: {}",
                snapshot.mmr_roots.keccak_root
            )
        })?,
        mmr_root_poseidon: FixedBytes::from_hex(&snapshot.mmr_roots.poseidon_root).map_err(
            |_| {
                anyhow!(
                    "invalid OP snapshot poseidon root for chain={name}, {variant}: {}",
                    snapshot.mmr_roots.poseidon_root
                )
            },
        )?,
    })
}

fn snapshot_mmr_root(
    snapshot: &OpChainClient,
    hashing_function: HashingFunction,
) -> FixedBytes<32> {
    match hashing_function {
        HashingFunction::Keccak => snapshot.mmr_root_keccak,
        HashingFunction::Poseidon => snapshot.mmr_root_poseidon,
    }
}

fn op_merkle_dto_to_input(dto: &OpMerkleProofDto) -> Result<OpStackMerkleProof> {
    let leaf_hash = FixedBytes::from_hex(&dto.leaf_hash)
        .map_err(|_| anyhow!("invalid OP leaf_hash {}", dto.leaf_hash))?;
    let root =
        FixedBytes::from_hex(&dto.root).map_err(|_| anyhow!("invalid OP root {}", dto.root))?;
    let path = dto
        .path
        .iter()
        .map(|item| FixedBytes::from_hex(item).map_err(|_| anyhow!("invalid OP path {item}")))
        .collect::<Result<Vec<_>>>()?;

    Ok(OpStackMerkleProof {
        chain_id: dto.chain_id,
        merkle_leaf_index: dto.merkle_leaf_index,
        leaf_hash,
        root,
        path,
    })
}

fn assert_block_proof_payload_matches(
    payload: &bankai_types::api::proofs::BlockProofPayloadDto,
    expected: ProofFormat,
    variant: &str,
) -> Result<()> {
    match (expected, payload) {
        (ProofFormat::Bin, bankai_types::api::proofs::BlockProofPayloadDto::Bin(_)) => Ok(()),
        (ProofFormat::Json, bankai_types::api::proofs::BlockProofPayloadDto::Json(_)) => Ok(()),
        (expected, actual) => Err(anyhow!(
            "proof payload format mismatch for {variant}: expected {:?}, got {:?}",
            expected,
            actual
        )),
    }
}
