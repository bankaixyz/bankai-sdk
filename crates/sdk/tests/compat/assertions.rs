use anyhow::{anyhow, Result};
use bankai_sdk::errors::SdkError;
use bankai_types::api::ethereum::{BeaconSnapshotDto, ExecutionSnapshotDto, MmrSnapshotDto};

pub fn assert_selector_height_ordering(
    label: &str,
    latest: u64,
    justified: u64,
    finalized: u64,
) -> Result<()> {
    if latest < justified {
        return Err(anyhow!(
            "{label}: expected latest ({latest}) >= justified ({justified})"
        ));
    }
    if justified < finalized {
        return Err(anyhow!(
            "{label}: expected justified ({justified}) >= finalized ({finalized})"
        ));
    }
    Ok(())
}

pub fn assert_beacon_snapshot_invariants(label: &str, snapshot: &BeaconSnapshotDto) -> Result<()> {
    assert_snapshot_bounds(
        label,
        snapshot.start_height,
        snapshot.finalized_height,
        snapshot.justified_height,
        snapshot.end_height,
    )?;
    assert_mmr_snapshot(label, &snapshot.mmr_snapshot)
}

pub fn assert_execution_snapshot_invariants(
    label: &str,
    snapshot: &ExecutionSnapshotDto,
) -> Result<()> {
    assert_snapshot_bounds(
        label,
        snapshot.start_height,
        snapshot.finalized_height,
        snapshot.justified_height,
        snapshot.end_height,
    )?;
    assert_mmr_snapshot(label, &snapshot.mmr_snapshot)
}

fn assert_snapshot_bounds(
    label: &str,
    start_height: u64,
    finalized_height: u64,
    justified_height: u64,
    end_height: u64,
) -> Result<()> {
    if start_height > end_height {
        return Err(anyhow!(
            "{label}: expected start_height ({start_height}) <= end_height ({end_height})"
        ));
    }
    if finalized_height > justified_height {
        return Err(anyhow!(
            "{label}: expected finalized_height ({finalized_height}) <= justified_height ({justified_height})"
        ));
    }
    if finalized_height > end_height {
        return Err(anyhow!(
            "{label}: expected finalized_height ({finalized_height}) <= end_height ({end_height})"
        ));
    }
    if justified_height > end_height {
        return Err(anyhow!(
            "{label}: expected justified_height ({justified_height}) <= end_height ({end_height})"
        ));
    }
    Ok(())
}

fn assert_mmr_snapshot(label: &str, snapshot: &MmrSnapshotDto) -> Result<()> {
    if snapshot.elements_count == 0 {
        return Err(anyhow!("{label}: expected mmr elements_count > 0"));
    }
    if snapshot.leafs_count == 0 {
        return Err(anyhow!("{label}: expected mmr leafs_count > 0"));
    }
    if snapshot.keccak_peaks.is_empty() {
        return Err(anyhow!(
            "{label}: expected mmr keccak_peaks to be non-empty"
        ));
    }
    if snapshot.poseidon_peaks.is_empty() {
        return Err(anyhow!(
            "{label}: expected mmr poseidon_peaks to be non-empty"
        ));
    }
    Ok(())
}

pub fn expect_api_error_response(err: SdkError, label: &str) -> Result<()> {
    match err {
        SdkError::ApiErrorResponse { .. } => Ok(()),
        other => Err(anyhow!("{label}: expected ApiErrorResponse, got {other}")),
    }
}
