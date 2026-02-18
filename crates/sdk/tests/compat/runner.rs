use std::collections::BTreeMap;

use alloy_primitives::FixedBytes;
use alloy_primitives::hex::FromHex;
use anyhow::{Context, Result, anyhow};
use bankai_sdk::errors::SdkError;
use bankai_sdk::parse_block_proof_payload;
use bankai_types::api::blocks::{BankaiMmrProofRequestDto, LatestBlockQueryDto};
use bankai_types::api::error::ErrorResponse;
use bankai_types::api::proofs::{
    BankaiBlockProofWithMmrDto, BlockProofPayloadDto, LightClientProofDto, MmrProofDto,
    ProofFormatDto,
};
use bankai_types::fetch::evm::MmrProof;
use bankai_types::proofs::BankaiMmrProofDto;
use bankai_types::{block::BankaiBlockOutput, proofs::BankaiBlockProofDto};
use bankai_verify::bankai::mmr::MmrVerifier;
use bankai_verify::bankai::stwo::verify_stwo_proof_hash_output;

use crate::compat::case::{
    ApiErrorSource, BankaiMmrProofSource, CompatArea, CompatCaseDef, CompatCaseId, CompatKind,
    DecodeAs, HttpMethod, LightClientProofSource, MmrProofSource, ProofHashSource, RawBodySource,
    SdkCallSpec,
};
use crate::compat::context::CompatContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuitePhase {
    Decode,
    Verify,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaseStatus {
    Passed,
    Skipped,
    Failed,
}

#[derive(Debug, Clone)]
pub struct CaseReport {
    pub id: CompatCaseId,
    pub area: CompatArea,
    pub required: bool,
    pub status: CaseStatus,
    pub detail: String,
}

pub async fn run_case(ctx: &CompatContext, case: CompatCaseDef) -> CaseReport {
    let result = run_case_inner(ctx, case).await;

    match result {
        Ok(()) => CaseReport {
            id: case.id,
            area: case.area,
            required: case.required,
            status: CaseStatus::Passed,
            detail: "ok".to_string(),
        },
        Err(err) if !case.required => {
            let debug_curl = debug_curl_for_case(ctx, &case).await;
            CaseReport {
                id: case.id,
                area: case.area,
                required: case.required,
                status: CaseStatus::Skipped,
                detail: format!("{err:#}\nrepro:\n{debug_curl}"),
            }
        }
        Err(err) => {
            let debug_curl = debug_curl_for_case(ctx, &case).await;
            CaseReport {
                id: case.id,
                area: case.area,
                required: case.required,
                status: CaseStatus::Failed,
                detail: format!("{err:#}\nrepro:\n{debug_curl}"),
            }
        }
    }
}

async fn run_case_inner(ctx: &CompatContext, case: CompatCaseDef) -> Result<()> {
    match case.kind {
        CompatKind::SdkCallDecode { call } => run_sdk_decode(ctx, call).await,
        CompatKind::RawHttpDecode {
            method,
            path,
            query,
            body,
            decode_as,
        } => run_raw_decode(ctx, method, path, query, body, decode_as).await,
        CompatKind::ProofHashConsistency { source } => {
            run_proof_hash_consistency(ctx, source).await
        }
        CompatKind::MmrProofVerify { source } => run_mmr_verify(ctx, source).await,
        CompatKind::BankaiMmrProofVerify { source } => run_bankai_mmr_verify(ctx, source).await,
        CompatKind::LightClientProofVerify { source } => {
            run_light_client_proof_verify(ctx, source).await
        }
        CompatKind::ApiErrorShape { source } => run_api_error_shape(ctx, source).await,
    }
}

pub fn case_in_phase(case: &CompatCaseDef, phase: SuitePhase) -> bool {
    match phase {
        SuitePhase::Decode => matches!(
            case.kind,
            CompatKind::SdkCallDecode { .. }
                | CompatKind::RawHttpDecode { .. }
                | CompatKind::ApiErrorShape { .. }
        ),
        SuitePhase::Verify => matches!(
            case.kind,
            CompatKind::ProofHashConsistency { .. }
                | CompatKind::MmrProofVerify { .. }
                | CompatKind::BankaiMmrProofVerify { .. }
                | CompatKind::LightClientProofVerify { .. }
        ),
    }
}

async fn run_sdk_decode(ctx: &CompatContext, call: SdkCallSpec) -> Result<()> {
    match call {
        SdkCallSpec::HealthGet => {
            let _ = ctx
                .sdk
                .api
                .health()
                .get()
                .await
                .context("health get failed")?;
        }
        SdkCallSpec::ChainsList => {
            let _ = ctx
                .sdk
                .api
                .chains()
                .list()
                .await
                .context("chains list failed")?;
        }
        SdkCallSpec::ChainsByIdFromList => {
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
            let _ = ctx
                .sdk
                .api
                .blocks()
                .list(&Default::default())
                .await
                .context("blocks list failed")?;
        }
        SdkCallSpec::BlocksLatestCompleted => {
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
                .proof_with_format(latest, ProofFormatDto::Bin)
                .await
                .context("blocks proof_with_format failed")?;
        }
        SdkCallSpec::BlocksProofByHeightFromLatest => {
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
        SdkCallSpec::BlocksMmrProofFromLatest => {
            let request = ctx.bankai_mmr_request_from_latest().await?;
            let _ = ctx
                .sdk
                .api
                .blocks()
                .mmr_proof(&request)
                .await
                .context("blocks mmr_proof failed")?;
        }
        SdkCallSpec::BlocksBlockProofFromLatest => {
            let request = ctx.bankai_block_proof_request_from_latest().await?;
            let _ = ctx
                .sdk
                .api
                .blocks()
                .block_proof(&request)
                .await
                .context("blocks block_proof failed")?;
        }
        SdkCallSpec::StatsOverview => {
            let _ = ctx
                .sdk
                .api
                .stats()
                .overview()
                .await
                .context("stats overview failed")?;
        }
        SdkCallSpec::StatsBlockDetailFromLatest => {
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
        SdkCallSpec::EthereumEpochFinalized => {
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .epoch(&ctx.finalized_filter())
                .await
                .context("ethereum epoch failed")?;
        }
        SdkCallSpec::EthereumEpochByNumberFromEpoch => {
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
        SdkCallSpec::EthereumBeaconHeightFinalized => {
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .height(&ctx.finalized_filter())
                .await
                .context("beacon height failed")?;
        }
        SdkCallSpec::EthereumBeaconSnapshotFinalized => {
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .snapshot(&ctx.finalized_filter())
                .await
                .context("beacon snapshot failed")?;
        }
        SdkCallSpec::EthereumBeaconMmrRootFinalized => {
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .mmr_root(&ctx.finalized_filter())
                .await
                .context("beacon mmr_root failed")?;
        }
        SdkCallSpec::EthereumBeaconMmrProofFromSnapshot => {
            let request = ctx.beacon_mmr_proof_request().await?;
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .mmr_proof(&request)
                .await
                .context("beacon mmr_proof failed")?;
        }
        SdkCallSpec::EthereumBeaconLightClientProofFromSnapshot => {
            let request = ctx.beacon_light_client_request().await?;
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .light_client_proof(&request)
                .await
                .context("beacon light_client_proof failed")?;
        }
        SdkCallSpec::EthereumExecutionHeightFinalized => {
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .height(&ctx.finalized_filter())
                .await
                .context("execution height failed")?;
        }
        SdkCallSpec::EthereumExecutionSnapshotFinalized => {
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .snapshot(&ctx.finalized_filter())
                .await
                .context("execution snapshot failed")?;
        }
        SdkCallSpec::EthereumExecutionMmrRootFinalized => {
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .mmr_root(&ctx.finalized_filter())
                .await
                .context("execution mmr_root failed")?;
        }
        SdkCallSpec::EthereumExecutionMmrProofFromSnapshot => {
            let request = ctx.execution_mmr_proof_request().await?;
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .mmr_proof(&request)
                .await
                .context("execution mmr_proof failed")?;
        }
        SdkCallSpec::EthereumExecutionLightClientProofFromSnapshot => {
            let request = ctx.execution_light_client_request().await?;
            let _ = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .light_client_proof(&request)
                .await
                .context("execution light_client_proof failed")?;
        }
    }

    Ok(())
}

async fn run_raw_decode(
    ctx: &CompatContext,
    method: HttpMethod,
    path: &'static str,
    query: &'static [(&'static str, &'static str)],
    body: Option<RawBodySource>,
    decode_as: DecodeAs,
) -> Result<()> {
    let value = send_raw_json(ctx, method, path, query, body).await?;

    match decode_as {
        DecodeAs::JsonValue => {
            let _: serde_json::Value = serde_json::from_value(value)
                .context("failed to decode raw payload as JSON value")?;
        }
        DecodeAs::BankaiBlockProofWithMmr => {
            let _ = parse_block_mmr_response(&value).with_context(|| {
                format!(
                    "failed to decode block mmr proof payload; response={}",
                    format_json_for_log(&value)
                )
            })?;
        }
        DecodeAs::BankaiBlockProofWithBlock => {
            let _ = parse_block_proof_with_block_response(&value).with_context(|| {
                format!(
                    "failed to decode block proof with block payload; response={}",
                    format_json_for_log(&value)
                )
            })?;
        }
    }

    Ok(())
}

async fn run_proof_hash_consistency(ctx: &CompatContext, source: ProofHashSource) -> Result<()> {
    match source {
        ProofHashSource::BlocksBlockProof => {
            let request = ctx.bankai_block_proof_request_from_latest().await?;
            let response = ctx
                .sdk
                .api
                .blocks()
                .block_proof(&request)
                .await
                .context("blocks block_proof failed")?;

            let block_proof = response.block_proof;
            let stwo_proof = parse_block_proof_payload(block_proof.proof)
                .context("failed to parse block proof payload")?;
            let _ = verify_stwo_proof_hash_output(stwo_proof)
                .context("STWO proof hash-output verification failed")?;

            let standalone_mmr_request = ctx.bankai_mmr_request_from_latest().await?;
            let standalone_mmr = ctx
                .sdk
                .api
                .blocks()
                .mmr_proof(&standalone_mmr_request)
                .await
                .context("blocks mmr_proof failed while checking block_proof consistency")?;

            validate_bankai_mmr_contract(&response.mmr_proof, &standalone_mmr_request)
                .context("block_proof returned invalid mmr_proof contract")?;
            assert_bankai_mmr_proofs_equal(&response.mmr_proof, &standalone_mmr)
                .context("block_proof mmr_proof does not match standalone /v1/blocks/mmr_proof")?;
        }
    }

    Ok(())
}

async fn run_mmr_verify(ctx: &CompatContext, source: MmrProofSource) -> Result<()> {
    let mmr: MmrProof = match source {
        MmrProofSource::EthereumBeaconFromSnapshot => {
            let request = ctx.beacon_mmr_proof_request().await?;
            let proof = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .mmr_proof(&request)
                .await
                .context("beacon mmr_proof failed")?;
            api_mmr_dto_to_mmr(&proof).context("failed converting beacon mmr proof")?
        }
        MmrProofSource::EthereumExecutionFromSnapshot => {
            let request = ctx.execution_mmr_proof_request().await?;
            let proof = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .mmr_proof(&request)
                .await
                .context("execution mmr_proof failed")?;
            api_mmr_dto_to_mmr(&proof).context("failed converting execution mmr proof")?
        }
    };

    let valid = MmrVerifier::verify_mmr_proof(&mmr).context("MMR proof verification failed")?;
    if !valid {
        return Err(anyhow!("MMR proof verifier returned false"));
    }

    Ok(())
}

async fn run_bankai_mmr_verify(ctx: &CompatContext, source: BankaiMmrProofSource) -> Result<()> {
    match source {
        BankaiMmrProofSource::BlocksMmrProofEndpoint => {
            let request = ctx.bankai_mmr_request_from_latest().await?;
            let mmr_proof = ctx
                .sdk
                .api
                .blocks()
                .mmr_proof(&request)
                .await
                .context("blocks mmr_proof failed")?;
            validate_bankai_mmr_contract(&mmr_proof, &request)
                .context("bankai mmr proof contract validation failed")?;
        }
    }

    Ok(())
}

async fn run_light_client_proof_verify(
    ctx: &CompatContext,
    source: LightClientProofSource,
) -> Result<()> {
    match source {
        LightClientProofSource::EthereumBeaconFromSnapshot => {
            let request = ctx.beacon_light_client_request().await?;
            let expected_root = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .mmr_root(&request.filter)
                .await
                .context("beacon mmr_root failed during light client verification")?
                .keccak_root;
            let proof = ctx
                .sdk
                .api
                .ethereum()
                .beacon()
                .light_client_proof(&request)
                .await
                .context("beacon light_client_proof failed")?;
            verify_light_client_bundle(&proof, &request.header_hashes, &expected_root)
                .context("beacon light client proof verification failed")?;
        }
        LightClientProofSource::EthereumExecutionFromSnapshot => {
            let request = ctx.execution_light_client_request().await?;
            let expected_root = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .mmr_root(&request.filter)
                .await
                .context("execution mmr_root failed during light client verification")?
                .keccak_root;
            let proof = ctx
                .sdk
                .api
                .ethereum()
                .execution()
                .light_client_proof(&request)
                .await
                .context("execution light_client_proof failed")?;
            verify_light_client_bundle(&proof, &request.header_hashes, &expected_root)
                .context("execution light client proof verification failed")?;
        }
    }
    Ok(())
}

fn verify_light_client_bundle(
    proof: &LightClientProofDto,
    requested_header_hashes: &[String],
    expected_mmr_root: &str,
) -> Result<()> {
    let stwo_proof = parse_block_proof_payload(proof.block_proof.proof.clone())
        .context("failed to parse block proof payload from light_client_proof")?;
    let _ = verify_stwo_proof_hash_output(stwo_proof)
        .context("STWO proof hash-output verification failed for light_client_proof")?;

    if proof.mmr_proofs.is_empty() {
        return Err(anyhow!("light_client_proof returned no mmr_proofs"));
    }

    for mmr_dto in &proof.mmr_proofs {
        if !requested_header_hashes
            .iter()
            .any(|header| hex_eq(header, &mmr_dto.header_hash))
        {
            return Err(anyhow!(
                "light_client_proof returned unexpected header hash {}",
                mmr_dto.header_hash
            ));
        }

        if !hex_eq(expected_mmr_root, &mmr_dto.root) {
            return Err(anyhow!(
                "light_client_proof root mismatch: expected {}, got {}",
                expected_mmr_root,
                mmr_dto.root
            ));
        }

        let mmr = api_mmr_dto_to_mmr(mmr_dto)
            .context("failed converting light client mmr proof to verifier type")?;
        let valid = MmrVerifier::verify_mmr_proof(&mmr)
            .context("light client mmr proof verification failed")?;
        if !valid {
            return Err(anyhow!("light client mmr verifier returned false"));
        }
    }

    Ok(())
}

async fn run_api_error_shape(ctx: &CompatContext, source: ApiErrorSource) -> Result<()> {
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
    }

    Ok(())
}

async fn send_raw_json(
    ctx: &CompatContext,
    method: HttpMethod,
    path: &'static str,
    query: &'static [(&'static str, &'static str)],
    body: Option<RawBodySource>,
) -> Result<serde_json::Value> {
    let url = ctx.url(path);
    let mut request = match method {
        HttpMethod::Get => ctx.http.get(&url),
        HttpMethod::Post => ctx.http.post(&url),
    };

    if !query.is_empty() {
        request = request.query(query);
    }

    if let Some(source) = body {
        let json = match source {
            RawBodySource::BankaiMmrProofRequestFromLatest => {
                ctx.raw_bankai_mmr_request_json().await?
            }
            RawBodySource::BankaiBlockProofRequestFromLatest => {
                ctx.raw_bankai_block_proof_request_json().await?
            }
        };
        request = request.json(&json);
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("request failed for {method:?} {url}"))?;

    let status = response.status();
    let body_text = response
        .text()
        .await
        .context("failed reading response body")?;

    if !status.is_success() {
        if let Ok(api_error) = serde_json::from_str::<ErrorResponse>(&body_text) {
            return Err(anyhow!(
                "raw endpoint returned api error {} ({}): {}",
                api_error.code,
                api_error.error_id,
                api_error.message,
            ));
        }
        return Err(anyhow!(
            "raw endpoint returned status {} with body: {}",
            status,
            body_text
        ));
    }

    let value = serde_json::from_str::<serde_json::Value>(&body_text)
        .with_context(|| format!("raw endpoint returned non-JSON payload: {body_text}"))?;

    Ok(value)
}

pub fn assert_reports(suite_name: &str, reports: &[CaseReport]) {
    #[derive(Default)]
    struct AreaSummary {
        required_total: usize,
        required_passed: usize,
        required_failed: usize,
        optional_total: usize,
        optional_passed: usize,
        optional_skipped: usize,
        optional_failed: usize,
        required_failure_ids: Vec<String>,
        optional_skip_ids: Vec<String>,
        optional_failure_ids: Vec<String>,
    }

    let mut required_total = 0usize;
    let mut required_passed = 0usize;
    let mut required_failed = 0usize;
    let mut optional_total = 0usize;
    let mut optional_passed = 0usize;
    let mut optional_skipped = 0usize;
    let mut optional_failed = 0usize;

    let mut by_area: BTreeMap<&'static str, AreaSummary> = BTreeMap::new();
    let mut required_failures: Vec<&CaseReport> = Vec::new();

    for report in reports {
        let area_key = area_name(report.area);
        let area = by_area.entry(area_key).or_default();

        if report.required {
            required_total += 1;
            area.required_total += 1;
            match report.status {
                CaseStatus::Passed => {
                    required_passed += 1;
                    area.required_passed += 1;
                }
                CaseStatus::Failed => {
                    required_failed += 1;
                    area.required_failed += 1;
                    area.required_failure_ids.push(report.id.0.to_string());
                    required_failures.push(report);
                }
                CaseStatus::Skipped => {
                    required_failed += 1;
                    area.required_failed += 1;
                    area.required_failure_ids.push(report.id.0.to_string());
                    required_failures.push(report);
                }
            }
        } else {
            optional_total += 1;
            area.optional_total += 1;
            match report.status {
                CaseStatus::Passed => {
                    optional_passed += 1;
                    area.optional_passed += 1;
                }
                CaseStatus::Skipped => {
                    optional_skipped += 1;
                    area.optional_skipped += 1;
                    area.optional_skip_ids.push(report.id.0.to_string());
                }
                CaseStatus::Failed => {
                    optional_failed += 1;
                    area.optional_failed += 1;
                    area.optional_failure_ids.push(report.id.0.to_string());
                }
            }
        }
    }

    let color = use_color();
    eprintln!(
        "{}",
        paint(color, "1;36", &format!("compat report ({suite_name})"))
    );
    eprintln!(
        "{}",
        format!(
            "required: {}/{} passed, {} failed",
            paint(color, "32", &required_passed.to_string()),
            required_total,
            if required_failed == 0 {
                paint(color, "32", "0")
            } else {
                paint(color, "31", &required_failed.to_string())
            }
        )
    );
    eprintln!(
        "{}",
        format!(
            "optional: {}/{} passed, {} skipped, {} failed",
            paint(color, "32", &optional_passed.to_string()),
            optional_total,
            if optional_skipped == 0 {
                paint(color, "32", "0")
            } else {
                paint(color, "33", &optional_skipped.to_string())
            },
            if optional_failed == 0 {
                paint(color, "32", "0")
            } else {
                paint(color, "31", &optional_failed.to_string())
            }
        )
    );
    eprintln!("{}", paint(color, "1", "by category:"));

    for (area, summary) in by_area {
        eprintln!(
            "- {}: required {}/{} pass ({} fail), optional {}/{} pass ({} skip, {} fail)",
            area,
            summary.required_passed,
            summary.required_total,
            summary.required_failed,
            summary.optional_passed,
            summary.optional_total,
            summary.optional_skipped,
            summary.optional_failed
        );
        if !summary.required_failure_ids.is_empty() {
            eprintln!(
                "  {} {}",
                paint(color, "31", "required failed cases:"),
                summary.required_failure_ids.join(", ")
            );
        }
        if !summary.optional_skip_ids.is_empty() {
            eprintln!(
                "  {} {}",
                paint(color, "33", "optional skipped cases:"),
                summary.optional_skip_ids.join(", ")
            );
        }
        if !summary.optional_failure_ids.is_empty() {
            eprintln!(
                "  {} {}",
                paint(color, "31", "optional failed cases:"),
                summary.optional_failure_ids.join(", ")
            );
        }
    }

    if !required_failures.is_empty() {
        let mut message = String::from("required compatibility cases failed:\n");
        for failure in required_failures {
            message.push_str(&format!(
                "- [{}] {}: {}\n",
                area_name(failure.area),
                failure.id.0,
                failure.detail
            ));
        }
        panic!("{message}");
    }
}

fn area_name(area: CompatArea) -> &'static str {
    match area {
        CompatArea::Health => "health",
        CompatArea::Chains => "chains",
        CompatArea::Blocks => "blocks",
        CompatArea::Stats => "stats",
        CompatArea::EthereumBeacon => "ethereum_beacon",
        CompatArea::EthereumExecution => "ethereum_execution",
        CompatArea::EthereumRoot => "ethereum_root",
    }
}

fn api_mmr_dto_to_mmr(dto: &MmrProofDto) -> Result<MmrProof> {
    let header_hash = FixedBytes::<32>::from_hex(&dto.header_hash)
        .with_context(|| format!("invalid header_hash {}", dto.header_hash))?;
    let root = FixedBytes::<32>::from_hex(&dto.root)
        .with_context(|| format!("invalid root {}", dto.root))?;
    let path = dto
        .path
        .iter()
        .map(|item| {
            FixedBytes::<32>::from_hex(item).with_context(|| format!("invalid path element {item}"))
        })
        .collect::<Result<Vec<_>>>()?;
    let peaks = dto
        .peaks
        .iter()
        .map(|item| {
            FixedBytes::<32>::from_hex(item).with_context(|| format!("invalid peak element {item}"))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(MmrProof {
        network_id: dto.network_id,
        block_number: dto.block_number,
        hashing_function: dto.hashing_function,
        header_hash,
        root,
        elements_index: dto.elements_index,
        elements_count: dto.elements_count,
        path,
        peaks,
    })
}

fn hex_eq(a: &str, b: &str) -> bool {
    normalize_hex(a) == normalize_hex(b)
}

fn normalize_hex(value: &str) -> String {
    value.trim_start_matches("0x").to_ascii_lowercase()
}

fn validate_bankai_mmr_contract(
    proof: &BankaiMmrProofDto,
    request: &BankaiMmrProofRequestDto,
) -> Result<()> {
    if let Some(expected_reference) = request.filter.bankai_block_number {
        if proof.reference_block_number != expected_reference {
            return Err(anyhow!(
                "reference_block_number mismatch: expected {}, got {}",
                expected_reference,
                proof.reference_block_number
            ));
        }
    }

    if let Some(expected_target) = request.target_block.block_number {
        if proof.target_block_number != expected_target {
            return Err(anyhow!(
                "target_block_number mismatch: expected {}, got {}",
                expected_target,
                proof.target_block_number
            ));
        }
    }

    if let Some(expected_hash) = request.target_block.block_hash.as_ref() {
        if !hex_eq(expected_hash, &proof.block_hash) {
            return Err(anyhow!(
                "target block_hash mismatch: expected {}, got {}",
                expected_hash,
                proof.block_hash
            ));
        }
    }

    if proof.hashing_function != request.hashing_function {
        return Err(anyhow!(
            "hashing_function mismatch: expected {:?}, got {:?}",
            request.hashing_function,
            proof.hashing_function
        ));
    }

    if proof.elements_count == 0 {
        return Err(anyhow!("elements_count must be > 0"));
    }
    if proof.elements_index == 0 || proof.elements_index > proof.elements_count {
        return Err(anyhow!(
            "elements_index {} must be within 1..={}",
            proof.elements_index,
            proof.elements_count
        ));
    }
    if proof.peaks.is_empty() {
        return Err(anyhow!("peaks must not be empty"));
    }

    let _ = FixedBytes::<32>::from_hex(&proof.block_hash)
        .with_context(|| format!("invalid block_hash {}", proof.block_hash))?;
    let _ = FixedBytes::<32>::from_hex(&proof.root)
        .with_context(|| format!("invalid root {}", proof.root))?;
    for item in &proof.path {
        let _ = FixedBytes::<32>::from_hex(item)
            .with_context(|| format!("invalid path element {item}"))?;
    }
    for item in &proof.peaks {
        let _ = FixedBytes::<32>::from_hex(item)
            .with_context(|| format!("invalid peak element {item}"))?;
    }

    Ok(())
}

fn assert_bankai_mmr_proofs_equal(a: &BankaiMmrProofDto, b: &BankaiMmrProofDto) -> Result<()> {
    if a.reference_block_number != b.reference_block_number {
        return Err(anyhow!(
            "reference_block_number mismatch: {} != {}",
            a.reference_block_number,
            b.reference_block_number
        ));
    }
    if a.target_block_number != b.target_block_number {
        return Err(anyhow!(
            "target_block_number mismatch: {} != {}",
            a.target_block_number,
            b.target_block_number
        ));
    }
    if a.hashing_function != b.hashing_function {
        return Err(anyhow!(
            "hashing_function mismatch: {:?} != {:?}",
            a.hashing_function,
            b.hashing_function
        ));
    }
    if !hex_eq(&a.block_hash, &b.block_hash) {
        return Err(anyhow!(
            "block_hash mismatch: {} != {}",
            a.block_hash,
            b.block_hash
        ));
    }
    if !hex_eq(&a.root, &b.root) {
        return Err(anyhow!("root mismatch: {} != {}", a.root, b.root));
    }
    if a.elements_index != b.elements_index {
        return Err(anyhow!(
            "elements_index mismatch: {} != {}",
            a.elements_index,
            b.elements_index
        ));
    }
    if a.elements_count != b.elements_count {
        return Err(anyhow!(
            "elements_count mismatch: {} != {}",
            a.elements_count,
            b.elements_count
        ));
    }
    if a.path.len() != b.path.len() {
        return Err(anyhow!(
            "path length mismatch: {} != {}",
            a.path.len(),
            b.path.len()
        ));
    }
    for (index, (left, right)) in a.path.iter().zip(b.path.iter()).enumerate() {
        if !hex_eq(left, right) {
            return Err(anyhow!("path[{index}] mismatch: {} != {}", left, right));
        }
    }
    if a.peaks.len() != b.peaks.len() {
        return Err(anyhow!(
            "peaks length mismatch: {} != {}",
            a.peaks.len(),
            b.peaks.len()
        ));
    }
    for (index, (left, right)) in a.peaks.iter().zip(b.peaks.iter()).enumerate() {
        if !hex_eq(left, right) {
            return Err(anyhow!("peaks[{index}] mismatch: {} != {}", left, right));
        }
    }
    Ok(())
}

fn parse_block_mmr_response(
    value: &serde_json::Value,
) -> Result<(BankaiBlockProofDto, BankaiMmrProofDto)> {
    if let Ok(decoded) = serde_json::from_value::<BankaiBlockProofWithMmrDto>(value.clone()) {
        return Ok((decoded.block_proof, decoded.mmr_proof));
    }

    let block_proof = parse_bankai_block_proof_dto(value)
        .context("missing block proof object (expected 'block_proof' or 'proof')")?;
    let mmr_value = value
        .get("mmr_proof")
        .cloned()
        .or_else(|| value.get("bankai_mmr_proof").cloned())
        .ok_or_else(|| anyhow!("missing mmr proof object (expected 'mmr_proof')"))?;
    let mmr_proof = serde_json::from_value::<BankaiMmrProofDto>(mmr_value)
        .context("failed to decode mmr proof object")?;
    Ok((block_proof, mmr_proof))
}

fn parse_block_proof_with_block_response(
    value: &serde_json::Value,
) -> Result<(BankaiBlockOutput, BankaiBlockProofDto)> {
    let block_proof = parse_bankai_block_proof_dto(value)
        .context("missing block proof object (expected 'block_proof' or 'proof')")?;

    let block_output =
        if let Ok(decoded) = serde_json::from_value::<BankaiBlockOutput>(value.clone()) {
            decoded
        } else if let Some(v) = value.get("block") {
            if let Ok(decoded) = serde_json::from_value::<BankaiBlockOutput>(v.clone()) {
                decoded
            } else if let Some(inner) = v.get("block") {
                let block_hash = v
                    .get("block_hash")
                    .or_else(|| value.get("block_hash"))
                    .ok_or_else(|| anyhow!("missing block_hash for nested block object"))?;
                let reconstructed = serde_json::json!({
                    "block_hash": block_hash,
                    "block": inner,
                });
                serde_json::from_value::<BankaiBlockOutput>(reconstructed)
                    .context("failed to decode nested block output")?
            } else if let Some(block_hash) = value.get("block_hash") {
                let reconstructed = serde_json::json!({
                    "block_hash": block_hash,
                    "block": v,
                });
                serde_json::from_value::<BankaiBlockOutput>(reconstructed)
                    .context("failed to decode block using top-level block_hash")?
            } else {
                return Err(anyhow!("failed to decode 'block' object"));
            }
        } else {
            return Err(anyhow!("failed to decode top-level block output"));
        };

    Ok((block_output, block_proof))
}

fn parse_bankai_block_proof_dto(value: &serde_json::Value) -> Result<BankaiBlockProofDto> {
    #[derive(serde::Deserialize)]
    struct BlockProofCompat {
        #[serde(default)]
        block_number: Option<u64>,
        proof: BlockProofPayloadDto,
    }

    if let Some(v) = value.get("block_proof") {
        if let Ok(decoded) = serde_json::from_value::<BankaiBlockProofDto>(v.clone()) {
            return Ok(decoded);
        }
        if let Ok(decoded) = serde_json::from_value::<BlockProofCompat>(v.clone()) {
            return Ok(BankaiBlockProofDto {
                block_number: decoded.block_number.unwrap_or_default(),
                proof: decoded.proof,
            });
        }
    }
    if let Some(v) = value.get("proof") {
        if let Ok(decoded) = serde_json::from_value::<BankaiBlockProofDto>(v.clone()) {
            return Ok(decoded);
        }
        if let Ok(decoded) = serde_json::from_value::<BlockProofCompat>(v.clone()) {
            return Ok(BankaiBlockProofDto {
                block_number: decoded.block_number.unwrap_or_default(),
                proof: decoded.proof,
            });
        }
        if let Ok(payload) = serde_json::from_value::<BlockProofPayloadDto>(v.clone()) {
            return Ok(BankaiBlockProofDto {
                block_number: 0,
                proof: payload,
            });
        }
    }
    if let Ok(decoded) = serde_json::from_value::<BlockProofCompat>(value.clone()) {
        return Ok(BankaiBlockProofDto {
            block_number: decoded.block_number.unwrap_or_default(),
            proof: decoded.proof,
        });
    }
    if let Ok(payload) = serde_json::from_value::<BlockProofPayloadDto>(value.clone()) {
        return Ok(BankaiBlockProofDto {
            block_number: 0,
            proof: payload,
        });
    }
    serde_json::from_value::<BankaiBlockProofDto>(value.clone())
        .context("failed to decode block proof object")
}

fn use_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::env::var("COMPAT_COLOR")
        .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true)
}

fn paint(enabled: bool, code: &str, text: &str) -> String {
    if enabled {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

async fn debug_curl_for_case(ctx: &CompatContext, case: &CompatCaseDef) -> String {
    match case.kind {
        CompatKind::RawHttpDecode {
            method,
            path,
            query,
            body,
            ..
        } => debug_curl_for_raw(ctx, method, path, query, body).await,
        CompatKind::ProofHashConsistency {
            source: ProofHashSource::BlocksBlockProof,
        } => {
            debug_curl_for_raw(
                ctx,
                HttpMethod::Post,
                "/v1/blocks/block_proof",
                &[],
                Some(RawBodySource::BankaiBlockProofRequestFromLatest),
            )
            .await
        }
        CompatKind::BankaiMmrProofVerify {
            source: BankaiMmrProofSource::BlocksMmrProofEndpoint,
        } => {
            debug_curl_for_raw(
                ctx,
                HttpMethod::Post,
                "/v1/blocks/mmr_proof",
                &[],
                Some(RawBodySource::BankaiMmrProofRequestFromLatest),
            )
            .await
        }
        CompatKind::MmrProofVerify { source } => debug_curl_for_mmr_verify(ctx, source).await,
        CompatKind::LightClientProofVerify { source } => {
            debug_curl_for_light_client_verify(ctx, source).await
        }
        CompatKind::SdkCallDecode { call } => debug_curl_for_sdk_call(ctx, call).await,
        CompatKind::ApiErrorShape { source } => debug_curl_for_api_error_shape(ctx, source).await,
    }
}

async fn debug_curl_for_raw(
    ctx: &CompatContext,
    method: HttpMethod,
    path: &'static str,
    query: &'static [(&'static str, &'static str)],
    body: Option<RawBodySource>,
) -> String {
    let url = format_url_with_query(ctx, path, query);
    let body_json = match body {
        Some(RawBodySource::BankaiMmrProofRequestFromLatest) => {
            ctx.raw_bankai_mmr_request_json().await.ok()
        }
        Some(RawBodySource::BankaiBlockProofRequestFromLatest) => {
            ctx.raw_bankai_block_proof_request_json().await.ok()
        }
        None => None,
    };
    match (body, body_json) {
        (Some(_), Some(json)) => build_curl_command(method, &url, Some(&json)),
        (Some(_), None) => format!(
            "{}\n# request body unavailable from test context; fetch latest completed block and build request JSON",
            build_curl_command(method, &url, None)
        ),
        (None, _) => build_curl_command(method, &url, None),
    }
}

async fn debug_curl_for_mmr_verify(ctx: &CompatContext, source: MmrProofSource) -> String {
    match source {
        MmrProofSource::EthereumBeaconFromSnapshot => {
            if let Ok(request) = ctx.beacon_mmr_proof_request().await {
                let url = ctx.url("/v1/ethereum/beacon/mmr_proof");
                build_curl_command(
                    HttpMethod::Post,
                    &url,
                    serde_json::to_value(request).ok().as_ref(),
                )
            } else {
                format!(
                    "{}\n{}",
                    "curl -sS 'http://<api>/v1/ethereum/beacon/snapshot?selector=finalized'",
                    "curl -sS -X POST 'http://<api>/v1/ethereum/beacon/mmr_proof' -H 'content-type: application/json' --data '<json-from-snapshot>'"
                )
            }
        }
        MmrProofSource::EthereumExecutionFromSnapshot => {
            if let Ok(request) = ctx.execution_mmr_proof_request().await {
                let url = ctx.url("/v1/ethereum/execution/mmr_proof");
                build_curl_command(
                    HttpMethod::Post,
                    &url,
                    serde_json::to_value(request).ok().as_ref(),
                )
            } else {
                format!(
                    "{}\n{}",
                    "curl -sS 'http://<api>/v1/ethereum/execution/snapshot?selector=finalized'",
                    "curl -sS -X POST 'http://<api>/v1/ethereum/execution/mmr_proof' -H 'content-type: application/json' --data '<json-from-snapshot>'"
                )
            }
        }
    }
}

async fn debug_curl_for_light_client_verify(
    ctx: &CompatContext,
    source: LightClientProofSource,
) -> String {
    match source {
        LightClientProofSource::EthereumBeaconFromSnapshot => {
            if let Ok(request) = ctx.beacon_light_client_request().await {
                let mmr_root = build_curl_command(
                    HttpMethod::Get,
                    &format_url_with_query(
                        ctx,
                        "/v1/ethereum/beacon/mmr_root",
                        &[("selector", "finalized")],
                    ),
                    None,
                );
                let lc = build_curl_command(
                    HttpMethod::Post,
                    &ctx.url("/v1/ethereum/beacon/light_client_proof"),
                    serde_json::to_value(request).ok().as_ref(),
                );
                format!("{mmr_root}\n{lc}")
            } else {
                format!(
                    "{}\n{}",
                    "curl -sS 'http://<api>/v1/ethereum/beacon/mmr_root?selector=finalized'",
                    "curl -sS -X POST 'http://<api>/v1/ethereum/beacon/light_client_proof' -H 'content-type: application/json' --data '<json-from-snapshot>'"
                )
            }
        }
        LightClientProofSource::EthereumExecutionFromSnapshot => {
            if let Ok(request) = ctx.execution_light_client_request().await {
                let mmr_root = build_curl_command(
                    HttpMethod::Get,
                    &format_url_with_query(
                        ctx,
                        "/v1/ethereum/execution/mmr_root",
                        &[("selector", "finalized")],
                    ),
                    None,
                );
                let lc = build_curl_command(
                    HttpMethod::Post,
                    &ctx.url("/v1/ethereum/execution/light_client_proof"),
                    serde_json::to_value(request).ok().as_ref(),
                );
                format!("{mmr_root}\n{lc}")
            } else {
                format!(
                    "{}\n{}",
                    "curl -sS 'http://<api>/v1/ethereum/execution/mmr_root?selector=finalized'",
                    "curl -sS -X POST 'http://<api>/v1/ethereum/execution/light_client_proof' -H 'content-type: application/json' --data '<json-from-snapshot>'"
                )
            }
        }
    }
}

async fn debug_curl_for_sdk_call(ctx: &CompatContext, call: SdkCallSpec) -> String {
    match call {
        SdkCallSpec::HealthGet => build_curl_command(HttpMethod::Get, &ctx.url("/v1/health"), None),
        SdkCallSpec::ChainsList => {
            build_curl_command(HttpMethod::Get, &ctx.url("/v1/chains"), None)
        }
        SdkCallSpec::ChainsByIdFromList => format!(
            "{}\n{}",
            build_curl_command(HttpMethod::Get, &ctx.url("/v1/chains"), None),
            "curl -sS 'http://<api>/v1/chains/<chain_id>'"
        ),
        SdkCallSpec::BlocksList => {
            build_curl_command(HttpMethod::Get, &ctx.url("/v1/blocks"), None)
        }
        SdkCallSpec::BlocksLatestCompleted => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(ctx, "/v1/blocks/latest", &[("status", "completed")]),
            None,
        ),
        SdkCallSpec::BlocksByHeightFromLatest => format!(
            "{}\n{}",
            build_curl_command(
                HttpMethod::Get,
                &format_url_with_query(ctx, "/v1/blocks/latest", &[("status", "completed")]),
                None
            ),
            "curl -sS 'http://<api>/v1/blocks/<height>'"
        ),
        SdkCallSpec::BlocksProofByQueryFromLatest => format!(
            "{}\n{}",
            build_curl_command(
                HttpMethod::Get,
                &format_url_with_query(ctx, "/v1/blocks/latest", &[("status", "completed")]),
                None
            ),
            "curl -sS 'http://<api>/v1/blocks/get_proof?block_number=<height>&proof_format=bin'"
        ),
        SdkCallSpec::BlocksProofByHeightFromLatest => format!(
            "{}\n{}",
            build_curl_command(
                HttpMethod::Get,
                &format_url_with_query(ctx, "/v1/blocks/latest", &[("status", "completed")]),
                None
            ),
            "curl -sS 'http://<api>/v1/blocks/<height>/proof?proof_format=bin'"
        ),
        SdkCallSpec::BlocksMmrProofFromLatest => {
            debug_curl_for_raw(
                ctx,
                HttpMethod::Post,
                "/v1/blocks/mmr_proof",
                &[],
                Some(RawBodySource::BankaiMmrProofRequestFromLatest),
            )
            .await
        }
        SdkCallSpec::BlocksBlockProofFromLatest => {
            debug_curl_for_raw(
                ctx,
                HttpMethod::Post,
                "/v1/blocks/block_proof",
                &[],
                Some(RawBodySource::BankaiBlockProofRequestFromLatest),
            )
            .await
        }
        SdkCallSpec::StatsOverview => {
            build_curl_command(HttpMethod::Get, &ctx.url("/v1/stats/overview"), None)
        }
        SdkCallSpec::StatsBlockDetailFromLatest => format!(
            "{}\n{}",
            build_curl_command(
                HttpMethod::Get,
                &format_url_with_query(ctx, "/v1/blocks/latest", &[("status", "completed")]),
                None
            ),
            "curl -sS 'http://<api>/v1/stats/block/<height>'"
        ),
        SdkCallSpec::EthereumEpochFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(ctx, "/v1/ethereum/epoch", &[("selector", "finalized")]),
            None,
        ),
        SdkCallSpec::EthereumEpochByNumberFromEpoch => format!(
            "{}\n{}",
            build_curl_command(
                HttpMethod::Get,
                &format_url_with_query(ctx, "/v1/ethereum/epoch", &[("selector", "finalized")]),
                None
            ),
            "curl -sS 'http://<api>/v1/ethereum/epoch/<epoch_number>'"
        ),
        SdkCallSpec::EthereumSyncCommitteeFromEpoch => format!(
            "{}\n{}",
            build_curl_command(
                HttpMethod::Get,
                &format_url_with_query(ctx, "/v1/ethereum/epoch", &[("selector", "finalized")]),
                None
            ),
            "curl -sS 'http://<api>/v1/ethereum/sync_committee?term_id=<term_id>'"
        ),
        SdkCallSpec::EthereumBeaconHeightFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(
                ctx,
                "/v1/ethereum/beacon/height",
                &[("selector", "finalized")],
            ),
            None,
        ),
        SdkCallSpec::EthereumBeaconSnapshotFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(
                ctx,
                "/v1/ethereum/beacon/snapshot",
                &[("selector", "finalized")],
            ),
            None,
        ),
        SdkCallSpec::EthereumBeaconMmrRootFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(
                ctx,
                "/v1/ethereum/beacon/mmr_root",
                &[("selector", "finalized")],
            ),
            None,
        ),
        SdkCallSpec::EthereumBeaconMmrProofFromSnapshot => {
            debug_curl_for_mmr_verify(ctx, MmrProofSource::EthereumBeaconFromSnapshot).await
        }
        SdkCallSpec::EthereumBeaconLightClientProofFromSnapshot => {
            debug_curl_for_light_client_verify(
                ctx,
                LightClientProofSource::EthereumBeaconFromSnapshot,
            )
            .await
        }
        SdkCallSpec::EthereumExecutionHeightFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(
                ctx,
                "/v1/ethereum/execution/height",
                &[("selector", "finalized")],
            ),
            None,
        ),
        SdkCallSpec::EthereumExecutionSnapshotFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(
                ctx,
                "/v1/ethereum/execution/snapshot",
                &[("selector", "finalized")],
            ),
            None,
        ),
        SdkCallSpec::EthereumExecutionMmrRootFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(
                ctx,
                "/v1/ethereum/execution/mmr_root",
                &[("selector", "finalized")],
            ),
            None,
        ),
        SdkCallSpec::EthereumExecutionMmrProofFromSnapshot => {
            debug_curl_for_mmr_verify(ctx, MmrProofSource::EthereumExecutionFromSnapshot).await
        }
        SdkCallSpec::EthereumExecutionLightClientProofFromSnapshot => {
            debug_curl_for_light_client_verify(
                ctx,
                LightClientProofSource::EthereumExecutionFromSnapshot,
            )
            .await
        }
    }
}

async fn debug_curl_for_api_error_shape(ctx: &CompatContext, source: ApiErrorSource) -> String {
    match source {
        ApiErrorSource::SyncCommitteeFromEpoch => {
            if let Ok(epoch) = ctx.epoch_from_finalized().await {
                if let Some(term_id) = epoch.sync_committee_term_id {
                    let url = format!(
                        "{}?term_id={}",
                        ctx.url("/v1/ethereum/sync_committee"),
                        term_id
                    );
                    return build_curl_command(HttpMethod::Get, &url, None);
                }
            }
            "curl -sS 'http://<api>/v1/ethereum/sync_committee?term_id=<term_id>'".to_string()
        }
    }
}

fn build_curl_command(
    method: HttpMethod,
    url: &str,
    body_json: Option<&serde_json::Value>,
) -> String {
    match method {
        HttpMethod::Get => format!("curl -sS '{}'", sh_single_quote(url)),
        HttpMethod::Post => {
            if let Some(body) = body_json {
                let body = serde_json::to_string(body).unwrap_or_else(|_| "{}".to_string());
                format!(
                    "curl -sS -X POST '{}' -H 'content-type: application/json' --data '{}'",
                    sh_single_quote(url),
                    sh_single_quote(&body)
                )
            } else {
                format!("curl -sS -X POST '{}'", sh_single_quote(url))
            }
        }
    }
}

fn format_url_with_query(
    ctx: &CompatContext,
    path: &'static str,
    query: &[(&str, &str)],
) -> String {
    if query.is_empty() {
        return ctx.url(path);
    }
    let qs = query
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{}?{}", ctx.url(path), qs)
}

fn sh_single_quote(value: &str) -> String {
    value.replace('\'', "'\"'\"'")
}

fn format_json_for_log(value: &serde_json::Value) -> String {
    let text = serde_json::to_string(value).unwrap_or_else(|_| "<invalid-json>".to_string());
    let max = 1200usize;
    if text.len() > max {
        format!("{}...(truncated)", &text[..max])
    } else {
        text
    }
}
