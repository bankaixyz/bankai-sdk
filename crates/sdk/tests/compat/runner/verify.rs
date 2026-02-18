use anyhow::{anyhow, Context, Result};
use bankai_sdk::parse_block_proof_payload;
use bankai_types::api::proofs::{LightClientProofDto, ProofFormatDto};
use bankai_types::fetch::evm::MmrProof;
use bankai_verify::bankai::mmr::MmrVerifier;
use bankai_verify::bankai::stwo::verify_stwo_proof_hash_output;

use crate::compat::case::{
    BankaiMmrProofSource, LightClientProofSource, MatrixScope, MmrProofSource, ProofHashSource,
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

                            let request = ctx.bankai_block_proof_request_for(
                                filter_case.filter.clone(),
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
                            let _ =
                                verify_stwo_proof_hash_output(stwo_proof).with_context(|| {
                                    format!(
                                        "STWO proof hash-output verification failed for {variant}"
                                    )
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
            };

            let valid = MmrVerifier::verify_mmr_proof(&mmr)
                .with_context(|| format!("MMR proof verification failed for {variant}"))?;
            if !valid {
                return Err(anyhow!("MMR verifier returned false for {variant}"));
            }
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
                }
            }
        }
    }

    Ok(())
}

fn verify_light_client_bundle(
    proof: &LightClientProofDto,
    requested_header_hashes: &[String],
    expected_mmr_root: &str,
    expected_hashing_function: bankai_types::api::proofs::HashingFunctionDto,
    variant: &str,
) -> Result<()> {
    let stwo_proof = parse_block_proof_payload(proof.block_proof.proof.clone())
        .with_context(|| format!("failed to parse block proof payload for {variant}"))?;
    let _ = verify_stwo_proof_hash_output(stwo_proof)
        .with_context(|| format!("STWO proof hash-output verification failed for {variant}"))?;

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
        let valid = MmrVerifier::verify_mmr_proof(&mmr)
            .with_context(|| format!("light client mmr proof verification failed for {variant}"))?;
        if !valid {
            return Err(anyhow!(
                "light client MMR verifier returned false for {variant}"
            ));
        }
    }

    Ok(())
}

fn assert_block_proof_payload_matches(
    payload: &bankai_types::api::proofs::BlockProofPayloadDto,
    expected: ProofFormatDto,
    variant: &str,
) -> Result<()> {
    match (expected, payload) {
        (ProofFormatDto::Bin, bankai_types::api::proofs::BlockProofPayloadDto::Bin(_)) => Ok(()),
        (ProofFormatDto::Json, bankai_types::api::proofs::BlockProofPayloadDto::Json(_)) => Ok(()),
        (expected, actual) => Err(anyhow!(
            "proof payload format mismatch for {variant}: expected {:?}, got {:?}",
            expected,
            actual
        )),
    }
}
