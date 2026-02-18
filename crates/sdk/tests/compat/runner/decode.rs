use anyhow::{anyhow, Context, Result};
use bankai_sdk::errors::SdkError;
use bankai_types::api::blocks::LatestBlockQueryDto;
use bankai_types::api::ethereum::BankaiBlockSelectorDto;
use bankai_types::api::proofs::BlockProofPayloadDto;

use crate::compat::assertions::{
    assert_beacon_snapshot_invariants, assert_execution_snapshot_invariants,
    assert_selector_height_ordering, expect_api_error_response,
};
use crate::compat::case::{ApiErrorSource, MatrixScope, SdkCallSpec};
use crate::compat::context::CompatContext;

use super::{format_variant, validate_bankai_mmr_contract, validate_mmr_proof_contract};

pub(super) async fn run_sdk_decode(
    ctx: &CompatContext,
    call: SdkCallSpec,
    scope: MatrixScope,
) -> Result<()> {
    match call {
        SdkCallSpec::HealthGet => {
            ensure_core_only(scope, "health.get")?;
            let _ = ctx
                .sdk
                .api
                .health()
                .get()
                .await
                .context("health get failed")?;
        }
        SdkCallSpec::ChainsList => {
            ensure_core_only(scope, "chains.list")?;
            let _ = ctx
                .sdk
                .api
                .chains()
                .list()
                .await
                .context("chains list failed")?;
        }
        SdkCallSpec::ChainsByIdFromList => {
            ensure_core_only(scope, "chains.by_id")?;
            let chains = ctx
                .sdk
                .api
                .chains()
                .list()
                .await
                .context("chains list failed")?;
            let chain = chains
                .first()
                .ok_or_else(|| anyhow!("chains list returned empty result"))?;
            let _ = ctx
                .sdk
                .api
                .chains()
                .by_id(chain.id)
                .await
                .context("chains by_id failed")?;
        }
        SdkCallSpec::BlocksList => {
            ensure_core_only(scope, "blocks.list")?;
            let _ = ctx
                .sdk
                .api
                .blocks()
                .list(&Default::default())
                .await
                .context("blocks list failed")?;
        }
        SdkCallSpec::BlocksLatestCompleted => {
            ensure_core_only(scope, "blocks.latest")?;
            let query = LatestBlockQueryDto {
                status: Some(bankai_types::api::blocks::BlockStatusDto::Completed),
            };
            let _ = ctx
                .sdk
                .api
                .blocks()
                .latest(&query)
                .await
                .context("blocks latest failed")?;
        }
        SdkCallSpec::BlocksByHeightFromLatest => {
            ensure_core_only(scope, "blocks.by_height")?;
            let latest = ctx
                .sdk
                .api
                .blocks()
                .latest_number()
                .await
                .context("latest block number failed")?;
            let _ = ctx
                .sdk
                .api
                .blocks()
                .by_height(latest)
                .await
                .context("blocks by_height failed")?;
        }
        SdkCallSpec::BlocksProofByQueryFromLatest => {
            ensure_core_only(scope, "blocks.proof_by_query")?;
            let latest = ctx
                .sdk
                .api
                .blocks()
                .latest_number()
                .await
                .context("latest block number failed")?;
            for proof_case in ctx.proof_format_cases_core() {
                let label = format_variant(&proof_case.label, scope);
                let proof = ctx
                    .sdk
                    .api
                    .blocks()
                    .proof_with_format(latest, proof_case.proof_format)
                    .await
                    .with_context(|| format!("blocks proof_with_format failed for {label}"))?;
                assert_block_proof_payload_format(
                    "blocks/get_proof",
                    &label,
                    &proof.proof,
                    proof_case.proof_format,
                )?;
            }
        }
        SdkCallSpec::BlocksProofByHeightFromLatest => {
            ensure_core_only(scope, "blocks.proof_by_height")?;
            let latest = ctx
                .sdk
                .api
                .blocks()
                .latest_number()
                .await
                .context("latest block number failed")?;
            let _ = ctx
                .sdk
                .api
                .blocks()
                .proof(latest)
                .await
                .context("blocks proof by height failed")?;
        }
        SdkCallSpec::BlocksMmrProofFromLatest => run_blocks_mmr_decode(ctx, scope).await?,
        SdkCallSpec::BlocksBlockProofFromLatest => {
            run_blocks_block_proof_decode(ctx, scope).await?
        }
        SdkCallSpec::StatsOverview => {
            ensure_core_only(scope, "stats.overview")?;
            let _ = ctx
                .sdk
                .api
                .stats()
                .overview()
                .await
                .context("stats overview failed")?;
        }
        SdkCallSpec::StatsBlockDetailFromLatest => {
            ensure_core_only(scope, "stats.block_detail")?;
            let latest = ctx
                .sdk
                .api
                .blocks()
                .latest_number()
                .await
                .context("latest block number failed")?;
            let _ = ctx
                .sdk
                .api
                .stats()
                .block_detail(latest)
                .await
                .context("stats block_detail failed")?;
        }
        SdkCallSpec::EthereumEpochFinalized => run_epoch_decode(ctx, scope).await?,
        SdkCallSpec::EthereumEpochByNumberFromEpoch => {
            ensure_core_only(scope, "ethereum.epoch_by_number")?;
            let epoch = ctx.epoch_from_finalized().await?;
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .epoch_by_number(epoch.number)
                .await
                .context("ethereum epoch_by_number failed")?;
        }
        SdkCallSpec::EthereumSyncCommitteeFromEpoch => {
            ensure_core_only(scope, "ethereum.sync_committee")?;
            let epoch = ctx.epoch_from_finalized().await?;
            let term_id = epoch
                .sync_committee_term_id
                .ok_or_else(|| anyhow!("epoch did not include sync_committee_term_id"))?;
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .sync_committee(term_id)
                .await
                .context("ethereum sync_committee failed")?;
        }
        SdkCallSpec::EthereumBeaconHeightFinalized => run_beacon_height_decode(ctx, scope).await?,
        SdkCallSpec::EthereumBeaconSnapshotFinalized => {
            run_beacon_snapshot_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumBeaconMmrRootFinalized => {
            run_beacon_mmr_root_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumBeaconMmrProofFromSnapshot => {
            run_beacon_mmr_proof_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumBeaconLightClientProofFromSnapshot => {
            run_beacon_light_client_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumExecutionHeightFinalized => {
            run_execution_height_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumExecutionSnapshotFinalized => {
            run_execution_snapshot_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumExecutionMmrRootFinalized => {
            run_execution_mmr_root_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumExecutionMmrProofFromSnapshot => {
            run_execution_mmr_proof_decode(ctx, scope).await?
        }
        SdkCallSpec::EthereumExecutionLightClientProofFromSnapshot => {
            run_execution_light_client_decode(ctx, scope).await?
        }
    }

    Ok(())
}

pub(super) async fn run_api_error_shape(
    ctx: &CompatContext,
    source: ApiErrorSource,
    scope: MatrixScope,
) -> Result<()> {
    match source {
        ApiErrorSource::SyncCommitteeFromEpoch => {
            let epoch = ctx.epoch_from_finalized().await?;
            let requested_term = epoch.sync_committee_term_id.unwrap_or(0);

            match ctx.sdk.api.ethereum().sync_committee(requested_term).await {
                Ok(_) => Ok(()),
                Err(SdkError::ApiErrorResponse { .. }) => Ok(()),
                Err(other) => Err(anyhow!(
                    "expected success or ApiErrorResponse for sync_committee; got {other}"
                )),
            }?;
        }
        ApiErrorSource::FilterConflict => {
            let conflict = ctx.conflicting_filter().await?;
            let res = ctx.sdk.api.ethereum().epoch(&conflict).await;
            expect_edge_api_error(
                res,
                &format_variant("ethereum.epoch filter conflict", scope),
            )?;
        }
    }

    Ok(())
}

async fn run_epoch_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };

    let mut latest = None;
    let mut justified = None;
    let mut finalized = None;

    for filter_case in filters {
        let label = format_variant(&filter_case.label, scope);
        let res = ctx.sdk.api.ethereum().epoch(&filter_case.filter).await;
        match scope {
            MatrixScope::Core => {
                let epoch = res.with_context(|| format!("ethereum epoch failed for {label}"))?;
                if let Some(selector) = filter_case.filter.selector {
                    match selector {
                        BankaiBlockSelectorDto::Latest => latest = Some(epoch.block_number),
                        BankaiBlockSelectorDto::Justified => justified = Some(epoch.block_number),
                        BankaiBlockSelectorDto::Finalized => finalized = Some(epoch.block_number),
                    }
                }
            }
            MatrixScope::Edge => {
                expect_edge_api_error(res, &label)?;
            }
        }
    }

    if scope == MatrixScope::Core {
        if let (Some(latest), Some(justified), Some(finalized)) = (latest, justified, finalized) {
            assert_selector_height_ordering("ethereum.epoch", latest, justified, finalized)?;
        }
    }

    Ok(())
}

async fn run_beacon_height_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };

    let mut latest = None;
    let mut justified = None;
    let mut finalized = None;

    for filter_case in filters {
        let label = format_variant(&filter_case.label, scope);
        let res = ctx
            .sdk
            .api
            .ethereum()
            .beacon()
            .height(&filter_case.filter)
            .await;
        match scope {
            MatrixScope::Core => {
                let height = res.with_context(|| format!("beacon height failed for {label}"))?;
                if let Some(selector) = filter_case.filter.selector {
                    match selector {
                        BankaiBlockSelectorDto::Latest => latest = Some(height.height),
                        BankaiBlockSelectorDto::Justified => justified = Some(height.height),
                        BankaiBlockSelectorDto::Finalized => finalized = Some(height.height),
                    }
                }
            }
            MatrixScope::Edge => expect_edge_api_error(res, &label)?,
        }
    }

    if scope == MatrixScope::Core {
        if let (Some(latest), Some(justified), Some(finalized)) = (latest, justified, finalized) {
            assert_selector_height_ordering(
                "ethereum.beacon.height",
                latest,
                justified,
                finalized,
            )?;
        }
    }

    Ok(())
}

async fn run_execution_height_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };

    let mut latest = None;
    let mut justified = None;
    let mut finalized = None;

    for filter_case in filters {
        let label = format_variant(&filter_case.label, scope);
        let res = ctx
            .sdk
            .api
            .ethereum()
            .execution()
            .height(&filter_case.filter)
            .await;
        match scope {
            MatrixScope::Core => {
                let height = res.with_context(|| format!("execution height failed for {label}"))?;
                if let Some(selector) = filter_case.filter.selector {
                    match selector {
                        BankaiBlockSelectorDto::Latest => latest = Some(height.height),
                        BankaiBlockSelectorDto::Justified => justified = Some(height.height),
                        BankaiBlockSelectorDto::Finalized => finalized = Some(height.height),
                    }
                }
            }
            MatrixScope::Edge => expect_edge_api_error(res, &label)?,
        }
    }

    if scope == MatrixScope::Core {
        if let (Some(latest), Some(justified), Some(finalized)) = (latest, justified, finalized) {
            assert_selector_height_ordering(
                "ethereum.execution.height",
                latest,
                justified,
                finalized,
            )?;
        }
    }

    Ok(())
}

async fn run_beacon_snapshot_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };

    for filter_case in filters {
        let label = format_variant(&filter_case.label, scope);
        let res = ctx
            .sdk
            .api
            .ethereum()
            .beacon()
            .snapshot(&filter_case.filter)
            .await;
        match scope {
            MatrixScope::Core => {
                let snapshot =
                    res.with_context(|| format!("beacon snapshot failed for {label}"))?;
                assert_beacon_snapshot_invariants("ethereum.beacon.snapshot", &snapshot)
                    .with_context(|| format!("snapshot invariants failed for {label}"))?;
            }
            MatrixScope::Edge => expect_edge_api_error(res, &label)?,
        }
    }

    Ok(())
}

async fn run_execution_snapshot_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };

    for filter_case in filters {
        let label = format_variant(&filter_case.label, scope);
        let res = ctx
            .sdk
            .api
            .ethereum()
            .execution()
            .snapshot(&filter_case.filter)
            .await;
        match scope {
            MatrixScope::Core => {
                let snapshot =
                    res.with_context(|| format!("execution snapshot failed for {label}"))?;
                assert_execution_snapshot_invariants("ethereum.execution.snapshot", &snapshot)
                    .with_context(|| format!("snapshot invariants failed for {label}"))?;
            }
            MatrixScope::Edge => expect_edge_api_error(res, &label)?,
        }
    }

    Ok(())
}

async fn run_beacon_mmr_root_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };

    for filter_case in filters {
        let label = format_variant(&filter_case.label, scope);
        let res = ctx
            .sdk
            .api
            .ethereum()
            .beacon()
            .mmr_root(&filter_case.filter)
            .await;
        match scope {
            MatrixScope::Core => {
                let roots = res.with_context(|| format!("beacon mmr_root failed for {label}"))?;
                if roots.keccak_root.is_empty() || roots.poseidon_root.is_empty() {
                    return Err(anyhow!("beacon mmr_root returned empty roots for {label}"));
                }
            }
            MatrixScope::Edge => expect_edge_api_error(res, &label)?,
        }
    }

    Ok(())
}

async fn run_execution_mmr_root_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
    let filters = match scope {
        MatrixScope::Core => ctx.filter_cases_core().await?,
        MatrixScope::Edge => ctx.filter_cases_edge().await?,
    };

    for filter_case in filters {
        let label = format_variant(&filter_case.label, scope);
        let res = ctx
            .sdk
            .api
            .ethereum()
            .execution()
            .mmr_root(&filter_case.filter)
            .await;
        match scope {
            MatrixScope::Core => {
                let roots =
                    res.with_context(|| format!("execution mmr_root failed for {label}"))?;
                if roots.keccak_root.is_empty() || roots.poseidon_root.is_empty() {
                    return Err(anyhow!(
                        "execution mmr_root returned empty roots for {label}"
                    ));
                }
            }
            MatrixScope::Edge => expect_edge_api_error(res, &label)?,
        }
    }

    Ok(())
}

async fn run_blocks_mmr_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
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

                let res = ctx.sdk.api.blocks().mmr_proof(&request).await;
                match scope {
                    MatrixScope::Core => {
                        let proof =
                            res.with_context(|| format!("blocks mmr_proof failed for {variant}"))?;
                        validate_bankai_mmr_contract(&proof, &request).with_context(|| {
                            format!("bankai mmr contract validation failed for {variant}")
                        })?;
                    }
                    MatrixScope::Edge => expect_edge_api_error(res, &variant)?,
                }
            }
        }
    }

    Ok(())
}

async fn run_blocks_block_proof_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
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

                    let res = ctx.sdk.api.blocks().block_proof(&request).await;
                    match scope {
                        MatrixScope::Core => {
                            let proof = res.with_context(|| {
                                format!("blocks block_proof failed for {variant}")
                            })?;
                            validate_bankai_mmr_contract(
                                &proof.mmr_proof,
                                &ctx.bankai_mmr_request_for(
                                    request.filter.clone(),
                                    request.target_block.clone(),
                                    request.hashing_function,
                                ),
                            )
                            .with_context(|| {
                                format!("bankai mmr contract validation failed for {variant}")
                            })?;
                            assert_block_proof_payload_format(
                                "blocks/block_proof",
                                &variant,
                                &proof.block_proof.proof,
                                proof_case.proof_format,
                            )?;
                        }
                        MatrixScope::Edge => expect_edge_api_error(res, &variant)?,
                    }
                }
            }
        }
    }

    Ok(())
}

async fn run_beacon_mmr_proof_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
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

            match scope {
                MatrixScope::Core => {
                    let request = ctx
                        .beacon_mmr_proof_request_for(
                            &filter_case.filter,
                            hashing_case.hashing_function,
                        )
                        .await
                        .with_context(|| {
                            format!("failed building beacon mmr request for {variant}")
                        })?;
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
                }
                MatrixScope::Edge => {
                    let mut request = ctx.beacon_mmr_proof_request().await?;
                    request.filter = filter_case.filter.clone();
                    request.hashing_function = hashing_case.hashing_function;
                    let res = ctx.sdk.api.ethereum().beacon().mmr_proof(&request).await;
                    expect_edge_api_error(res, &variant)?;
                }
            }
        }
    }

    Ok(())
}

async fn run_execution_mmr_proof_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
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

            match scope {
                MatrixScope::Core => {
                    let request = ctx
                        .execution_mmr_proof_request_for(
                            &filter_case.filter,
                            hashing_case.hashing_function,
                        )
                        .await
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
                }
                MatrixScope::Edge => {
                    let mut request = ctx.execution_mmr_proof_request().await?;
                    request.filter = filter_case.filter.clone();
                    request.hashing_function = hashing_case.hashing_function;
                    let res = ctx.sdk.api.ethereum().execution().mmr_proof(&request).await;
                    expect_edge_api_error(res, &variant)?;
                }
            }
        }
    }

    Ok(())
}

async fn run_beacon_light_client_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
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

                match scope {
                    MatrixScope::Core => {
                        let request = ctx
                            .beacon_light_client_request_for(
                                &filter_case.filter,
                                hashing_case.hashing_function,
                                proof_case.proof_format,
                            )
                            .await
                            .with_context(|| {
                                format!("failed building beacon light client request for {variant}")
                            })?;
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
                        if proof.mmr_proofs.is_empty() {
                            return Err(anyhow!(
                                "beacon light_client_proof returned no mmr_proofs for {variant}"
                            ));
                        }
                        for mmr in &proof.mmr_proofs {
                            if mmr.hashing_function != request.hashing_function {
                                return Err(anyhow!(
                                    "beacon light_client_proof hashing_function mismatch for {variant}: expected {:?}, got {:?}",
                                    request.hashing_function,
                                    mmr.hashing_function
                                ));
                            }
                        }
                        assert_block_proof_payload_format(
                            "ethereum/beacon/light_client_proof",
                            &variant,
                            &proof.block_proof.proof,
                            proof_case.proof_format,
                        )?;
                    }
                    MatrixScope::Edge => {
                        let mut request = ctx.beacon_light_client_request().await?;
                        request.filter = filter_case.filter.clone();
                        request.hashing_function = hashing_case.hashing_function;
                        request.proof_format = proof_case.proof_format;
                        let res = ctx
                            .sdk
                            .api
                            .ethereum()
                            .beacon()
                            .light_client_proof(&request)
                            .await;
                        expect_edge_api_error(res, &variant)?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn run_execution_light_client_decode(ctx: &CompatContext, scope: MatrixScope) -> Result<()> {
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

                match scope {
                    MatrixScope::Core => {
                        let request = ctx
                            .execution_light_client_request_for(
                                &filter_case.filter,
                                hashing_case.hashing_function,
                                proof_case.proof_format,
                            )
                            .await
                            .with_context(|| {
                                format!(
                                    "failed building execution light client request for {variant}"
                                )
                            })?;
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
                        if proof.mmr_proofs.is_empty() {
                            return Err(anyhow!(
                                "execution light_client_proof returned no mmr_proofs for {variant}"
                            ));
                        }
                        for mmr in &proof.mmr_proofs {
                            if mmr.hashing_function != request.hashing_function {
                                return Err(anyhow!(
                                    "execution light_client_proof hashing_function mismatch for {variant}: expected {:?}, got {:?}",
                                    request.hashing_function,
                                    mmr.hashing_function
                                ));
                            }
                        }
                        assert_block_proof_payload_format(
                            "ethereum/execution/light_client_proof",
                            &variant,
                            &proof.block_proof.proof,
                            proof_case.proof_format,
                        )?;
                    }
                    MatrixScope::Edge => {
                        let mut request = ctx.execution_light_client_request().await?;
                        request.filter = filter_case.filter.clone();
                        request.hashing_function = hashing_case.hashing_function;
                        request.proof_format = proof_case.proof_format;
                        let res = ctx
                            .sdk
                            .api
                            .ethereum()
                            .execution()
                            .light_client_proof(&request)
                            .await;
                        expect_edge_api_error(res, &variant)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn assert_block_proof_payload_format(
    endpoint: &str,
    label: &str,
    payload: &BlockProofPayloadDto,
    expected_format: bankai_types::api::proofs::ProofFormatDto,
) -> Result<()> {
    match (expected_format, payload) {
        (bankai_types::api::proofs::ProofFormatDto::Bin, BlockProofPayloadDto::Bin(_)) => Ok(()),
        (bankai_types::api::proofs::ProofFormatDto::Json, BlockProofPayloadDto::Json(_)) => Ok(()),
        (expected, actual) => Err(anyhow!(
            "{endpoint} returned payload format mismatch for {label}: expected {:?}, got {:?}",
            expected,
            actual
        )),
    }
}

fn ensure_core_only(scope: MatrixScope, case_name: &str) -> Result<()> {
    if scope == MatrixScope::Edge {
        return Err(anyhow!("{case_name} has no edge matrix variants"));
    }
    Ok(())
}

fn expect_edge_api_error<T>(result: Result<T, SdkError>, label: &str) -> Result<()> {
    match result {
        Ok(_) => Err(anyhow!(
            "expected ApiErrorResponse for optional edge variant `{label}`, but call succeeded"
        )),
        Err(err) => expect_api_error_response(err, label),
    }
}
