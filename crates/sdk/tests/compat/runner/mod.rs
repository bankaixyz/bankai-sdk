mod debug;
mod decode;
mod verify;

use std::collections::BTreeMap;

use alloy_primitives::hex::FromHex;
use alloy_primitives::FixedBytes;
use anyhow::{anyhow, Result};
use bankai_types::api::blocks::BankaiMmrProofRequestDto;
use bankai_types::api::proofs::MmrProofDto;
use bankai_types::fetch::evm::MmrProof;
use bankai_types::proofs::BankaiMmrProofDto;

use crate::compat::case::{
    ApiErrorSource, CompatArea, CompatCaseDef, CompatCaseId, CompatKind, MatrixScope, SdkCallSpec,
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
    pub matrix_variants: usize,
    pub status: CaseStatus,
    pub detail: String,
}

pub async fn run_case(ctx: &CompatContext, case: CompatCaseDef) -> CaseReport {
    let matrix_variants = planned_matrix_variants(&case);
    let result = run_case_inner(ctx, case).await;

    match result {
        Ok(()) => CaseReport {
            id: case.id,
            area: case.area,
            required: case.required,
            matrix_variants,
            status: CaseStatus::Passed,
            detail: "ok".to_string(),
        },
        Err(err) if !case.required => {
            let debug_curl = debug::debug_curl_for_case(ctx, &case).await;
            CaseReport {
                id: case.id,
                area: case.area,
                required: case.required,
                matrix_variants,
                status: CaseStatus::Skipped,
                detail: format!("{err:#}\nrepro:\n{debug_curl}"),
            }
        }
        Err(err) => {
            let debug_curl = debug::debug_curl_for_case(ctx, &case).await;
            CaseReport {
                id: case.id,
                area: case.area,
                required: case.required,
                matrix_variants,
                status: CaseStatus::Failed,
                detail: format!("{err:#}\nrepro:\n{debug_curl}"),
            }
        }
    }
}

async fn run_case_inner(ctx: &CompatContext, case: CompatCaseDef) -> Result<()> {
    match case.kind {
        CompatKind::SdkCallDecode { call, scope } => decode::run_sdk_decode(ctx, call, scope).await,
        CompatKind::ProofHashConsistency { source, scope } => {
            verify::run_proof_hash_consistency(ctx, source, scope).await
        }
        CompatKind::MmrProofVerify { source, scope } => {
            verify::run_mmr_verify(ctx, source, scope).await
        }
        CompatKind::BankaiMmrProofVerify { source, scope } => {
            verify::run_bankai_mmr_verify(ctx, source, scope).await
        }
        CompatKind::LightClientProofVerify { source, scope } => {
            verify::run_light_client_proof_verify(ctx, source, scope).await
        }
        CompatKind::ApiErrorShape { source, scope } => {
            decode::run_api_error_shape(ctx, source, scope).await
        }
    }
}

pub fn case_in_phase(case: &CompatCaseDef, phase: SuitePhase) -> bool {
    match phase {
        SuitePhase::Decode => matches!(
            case.kind,
            CompatKind::SdkCallDecode { .. } | CompatKind::ApiErrorShape { .. }
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

pub fn assert_reports(suite_name: &str, reports: &[CaseReport]) {
    #[derive(Default)]
    struct AreaSummary {
        required_total: usize,
        required_passed: usize,
        required_failed: usize,
        required_matrix_variants: usize,
        optional_total: usize,
        optional_passed: usize,
        optional_skipped: usize,
        optional_failed: usize,
        optional_matrix_variants: usize,
        required_failure_ids: Vec<String>,
        optional_skip_ids: Vec<String>,
        optional_failure_ids: Vec<String>,
    }

    let mut required_total = 0usize;
    let mut required_passed = 0usize;
    let mut required_failed = 0usize;
    let mut required_matrix_variants = 0usize;
    let mut optional_total = 0usize;
    let mut optional_passed = 0usize;
    let mut optional_skipped = 0usize;
    let mut optional_failed = 0usize;
    let mut optional_matrix_variants = 0usize;

    let mut by_area: BTreeMap<&'static str, AreaSummary> = BTreeMap::new();
    let mut required_failures: Vec<&CaseReport> = Vec::new();

    for report in reports {
        let area_key = area_name(report.area);
        let area = by_area.entry(area_key).or_default();

        if report.required {
            required_total += 1;
            area.required_total += 1;
            required_matrix_variants += report.matrix_variants;
            area.required_matrix_variants += report.matrix_variants;
            match report.status {
                CaseStatus::Passed => {
                    required_passed += 1;
                    area.required_passed += 1;
                }
                CaseStatus::Failed | CaseStatus::Skipped => {
                    required_failed += 1;
                    area.required_failed += 1;
                    area.required_failure_ids.push(report.id.0.to_string());
                    required_failures.push(report);
                }
            }
        } else {
            optional_total += 1;
            area.optional_total += 1;
            optional_matrix_variants += report.matrix_variants;
            area.optional_matrix_variants += report.matrix_variants;
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
    eprintln!(
        "{}",
        format!(
            "matrix variants (planned): {} total ({} required, {} optional)",
            required_matrix_variants + optional_matrix_variants,
            required_matrix_variants,
            optional_matrix_variants
        )
    );
    eprintln!("{}", paint(color, "1", "by category:"));

    for (area, summary) in by_area {
        eprintln!(
            "- {}: required {}/{} pass ({} fail), optional {}/{} pass ({} skip, {} fail), matrix {} req / {} opt",
            area,
            summary.required_passed,
            summary.required_total,
            summary.required_failed,
            summary.optional_passed,
            summary.optional_total,
            summary.optional_skipped,
            summary.optional_failed,
            summary.required_matrix_variants,
            summary.optional_matrix_variants
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

pub(super) fn area_name(area: CompatArea) -> &'static str {
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

pub(super) fn api_mmr_dto_to_mmr(dto: &MmrProofDto) -> Result<MmrProof> {
    let header_hash = FixedBytes::<32>::from_hex(&dto.header_hash)
        .map_err(|_| anyhow!("invalid header_hash {}", dto.header_hash))?;
    let root =
        FixedBytes::<32>::from_hex(&dto.root).map_err(|_| anyhow!("invalid root {}", dto.root))?;
    let path = dto
        .path
        .iter()
        .map(|item| {
            FixedBytes::<32>::from_hex(item).map_err(|_| anyhow!("invalid path element {item}"))
        })
        .collect::<Result<Vec<_>>>()?;
    let peaks = dto
        .peaks
        .iter()
        .map(|item| {
            FixedBytes::<32>::from_hex(item).map_err(|_| anyhow!("invalid peak element {item}"))
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

pub(super) fn validate_mmr_proof_contract(
    dto: &MmrProofDto,
    expected_header_hash: &str,
) -> Result<()> {
    if !hex_eq(&dto.header_hash, expected_header_hash) {
        return Err(anyhow!(
            "header_hash mismatch: expected {}, got {}",
            expected_header_hash,
            dto.header_hash
        ));
    }
    if dto.elements_count == 0 {
        return Err(anyhow!("elements_count must be > 0"));
    }
    if dto.elements_index == 0 || dto.elements_index > dto.elements_count {
        return Err(anyhow!(
            "elements_index {} must be within 1..={}",
            dto.elements_index,
            dto.elements_count
        ));
    }
    if dto.peaks.is_empty() {
        return Err(anyhow!("peaks must not be empty"));
    }
    Ok(())
}

pub(super) fn validate_bankai_mmr_contract(
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
                proof.block_hash,
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

    Ok(())
}

pub(super) fn assert_bankai_mmr_proofs_equal(
    a: &BankaiMmrProofDto,
    b: &BankaiMmrProofDto,
) -> Result<()> {
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

pub(super) fn format_variant(label: &str, scope: MatrixScope) -> String {
    match scope {
        MatrixScope::Core => label.to_string(),
        MatrixScope::Edge => format!("{label} [edge]"),
    }
}

pub(super) fn hex_eq(a: &str, b: &str) -> bool {
    normalize_hex(a) == normalize_hex(b)
}

pub(super) fn normalize_hex(value: &str) -> String {
    value.trim_start_matches("0x").to_ascii_lowercase()
}

fn planned_matrix_variants(case: &CompatCaseDef) -> usize {
    const FILTERS_CORE: usize = 4;
    const FILTERS_EDGE: usize = 1;
    const TARGETS_CORE: usize = 2;
    const TARGETS_EDGE: usize = 1;
    const HASHINGS_CORE: usize = 1;
    const HASHINGS_EDGE: usize = 1;
    const PROOF_FORMATS_CORE: usize = 2;

    match case.kind {
        CompatKind::SdkCallDecode { call, scope } => match call {
            SdkCallSpec::HealthGet
            | SdkCallSpec::ChainsList
            | SdkCallSpec::ChainsByIdFromList
            | SdkCallSpec::BlocksList
            | SdkCallSpec::BlocksLatestCompleted
            | SdkCallSpec::BlocksByHeightFromLatest
            | SdkCallSpec::BlocksProofByHeightFromLatest
            | SdkCallSpec::StatsOverview
            | SdkCallSpec::StatsBlockDetailFromLatest
            | SdkCallSpec::EthereumEpochByNumberFromEpoch
            | SdkCallSpec::EthereumSyncCommitteeFromEpoch => 1,
            SdkCallSpec::BlocksProofByQueryFromLatest => PROOF_FORMATS_CORE,
            SdkCallSpec::EthereumEpochFinalized => match scope {
                MatrixScope::Core => FILTERS_CORE,
                MatrixScope::Edge => FILTERS_EDGE,
            },
            SdkCallSpec::EthereumBeaconHeightFinalized
            | SdkCallSpec::EthereumBeaconSnapshotFinalized
            | SdkCallSpec::EthereumBeaconMmrRootFinalized
            | SdkCallSpec::EthereumExecutionHeightFinalized
            | SdkCallSpec::EthereumExecutionSnapshotFinalized
            | SdkCallSpec::EthereumExecutionMmrRootFinalized => match scope {
                MatrixScope::Core => FILTERS_CORE,
                MatrixScope::Edge => FILTERS_EDGE,
            },
            SdkCallSpec::BlocksMmrProofFromLatest => match scope {
                MatrixScope::Core => FILTERS_CORE * TARGETS_CORE * HASHINGS_CORE,
                MatrixScope::Edge => FILTERS_EDGE * TARGETS_EDGE * HASHINGS_EDGE,
            },
            SdkCallSpec::BlocksBlockProofFromLatest => match scope {
                MatrixScope::Core => {
                    FILTERS_CORE * TARGETS_CORE * HASHINGS_CORE * PROOF_FORMATS_CORE
                }
                MatrixScope::Edge => {
                    FILTERS_EDGE * TARGETS_EDGE * HASHINGS_EDGE * PROOF_FORMATS_CORE
                }
            },
            SdkCallSpec::EthereumBeaconMmrProofFromSnapshot
            | SdkCallSpec::EthereumExecutionMmrProofFromSnapshot => match scope {
                MatrixScope::Core => FILTERS_CORE * HASHINGS_CORE,
                MatrixScope::Edge => FILTERS_EDGE * HASHINGS_EDGE,
            },
            SdkCallSpec::EthereumBeaconLightClientProofFromSnapshot
            | SdkCallSpec::EthereumExecutionLightClientProofFromSnapshot => match scope {
                MatrixScope::Core => FILTERS_CORE * HASHINGS_CORE * PROOF_FORMATS_CORE,
                MatrixScope::Edge => FILTERS_EDGE * HASHINGS_EDGE * PROOF_FORMATS_CORE,
            },
        },
        CompatKind::ProofHashConsistency { scope, .. } => match scope {
            MatrixScope::Core => FILTERS_CORE * TARGETS_CORE * HASHINGS_CORE * PROOF_FORMATS_CORE,
            MatrixScope::Edge => FILTERS_EDGE * TARGETS_EDGE * HASHINGS_EDGE * PROOF_FORMATS_CORE,
        },
        CompatKind::MmrProofVerify { scope, .. } => match scope {
            MatrixScope::Core => FILTERS_CORE * HASHINGS_CORE,
            MatrixScope::Edge => FILTERS_EDGE * HASHINGS_EDGE,
        },
        CompatKind::BankaiMmrProofVerify { scope, .. } => match scope {
            MatrixScope::Core => FILTERS_CORE * TARGETS_CORE * HASHINGS_CORE,
            MatrixScope::Edge => FILTERS_EDGE * TARGETS_EDGE * HASHINGS_EDGE,
        },
        CompatKind::LightClientProofVerify { scope, .. } => match scope {
            MatrixScope::Core => FILTERS_CORE * HASHINGS_CORE * PROOF_FORMATS_CORE,
            MatrixScope::Edge => FILTERS_EDGE * HASHINGS_EDGE * PROOF_FORMATS_CORE,
        },
        CompatKind::ApiErrorShape { source, .. } => match source {
            ApiErrorSource::SyncCommitteeFromEpoch => 1,
            ApiErrorSource::FilterConflict => 1,
        },
    }
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
