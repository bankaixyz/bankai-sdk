use std::collections::{BTreeSet, HashSet};

use anyhow::{anyhow, Context, Result};

use crate::compat::context::CompatContext;

const IGNORED_PATHS: &[&str] = &["/v1/openapi.json", "/v1/swagger", "/v1/rapidoc"];

const ENDPOINT_CASE_MAP: &[(&str, &str, &str)] = &[
    ("get", "/v1/health", "health.get.decode"),
    ("get", "/v1/chains", "chains.list.decode"),
    ("get", "/v1/chains/{chain_id}", "chains.by_id.decode"),
    ("get", "/v1/blocks", "blocks.list.decode"),
    ("get", "/v1/blocks/latest", "blocks.latest.decode"),
    ("get", "/v1/blocks/{height}", "blocks.by_height.decode"),
    (
        "get",
        "/v1/blocks/get_proof",
        "blocks.proof_by_query.decode",
    ),
    (
        "get",
        "/v1/blocks/{height}/proof",
        "blocks.proof_by_height.decode",
    ),
    ("post", "/v1/blocks/mmr_proof", "blocks.mmr_proof.decode"),
    (
        "post",
        "/v1/blocks/block_proof",
        "blocks.block_proof.decode",
    ),
    ("get", "/v1/stats/overview", "stats.overview.decode"),
    (
        "get",
        "/v1/stats/block/{height}",
        "stats.block_detail.decode",
    ),
    ("get", "/v1/ethereum/epoch", "ethereum.epoch.decode"),
    (
        "get",
        "/v1/ethereum/epoch/{number}",
        "ethereum.epoch_by_number.decode",
    ),
    (
        "get",
        "/v1/ethereum/sync_committee",
        "ethereum.sync_committee.error_shape",
    ),
    (
        "get",
        "/v1/ethereum/beacon/height",
        "ethereum.beacon.height.decode",
    ),
    (
        "get",
        "/v1/ethereum/beacon/snapshot",
        "ethereum.beacon.snapshot.decode",
    ),
    (
        "get",
        "/v1/ethereum/beacon/mmr_root",
        "ethereum.beacon.mmr_root.decode",
    ),
    (
        "post",
        "/v1/ethereum/beacon/mmr_proof",
        "ethereum.beacon.mmr_proof.decode",
    ),
    (
        "post",
        "/v1/ethereum/beacon/light_client_proof",
        "ethereum.beacon.light_client_proof.decode",
    ),
    (
        "get",
        "/v1/ethereum/execution/height",
        "ethereum.execution.height.decode",
    ),
    (
        "get",
        "/v1/ethereum/execution/snapshot",
        "ethereum.execution.snapshot.decode",
    ),
    (
        "get",
        "/v1/ethereum/execution/mmr_root",
        "ethereum.execution.mmr_root.decode",
    ),
    (
        "post",
        "/v1/ethereum/execution/mmr_proof",
        "ethereum.execution.mmr_proof.decode",
    ),
    (
        "post",
        "/v1/ethereum/execution/light_client_proof",
        "ethereum.execution.light_client_proof.decode",
    ),
];

pub async fn run(ctx: &CompatContext) -> Result<()> {
    let url = ctx.url("/v1/openapi.json");
    let response = ctx
        .http
        .get(&url)
        .send()
        .await
        .with_context(|| format!("failed to fetch openapi spec from {url}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("failed reading openapi body")?;

    if !status.is_success() {
        return Err(anyhow!(
            "openapi endpoint returned status {} with body: {}",
            status,
            body
        ));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&body).context("openapi payload is not valid JSON")?;
    let paths = parsed
        .get("paths")
        .and_then(|value| value.as_object())
        .ok_or_else(|| anyhow!("openapi JSON is missing object field 'paths'"))?;

    let documented = documented_endpoints(paths)?;
    let (mapped, missing_case_ids) = mapped_endpoints();

    if !missing_case_ids.is_empty() {
        return Err(anyhow!(
            "endpoint coverage map references unknown compat case ids:\n{}",
            missing_case_ids.join("\n")
        ));
    }

    let missing_from_map = diff_endpoints(&documented, &mapped);
    if !missing_from_map.is_empty() {
        return Err(anyhow!(
            "openapi contains endpoints without SDK compat mapping:\n{}",
            missing_from_map.join("\n")
        ));
    }

    let stale_mappings = diff_endpoints(&mapped, &documented);
    if !stale_mappings.is_empty() {
        return Err(anyhow!(
            "SDK endpoint coverage map contains endpoints not present in openapi:\n{}",
            stale_mappings.join("\n")
        ));
    }

    Ok(())
}

fn documented_endpoints(
    paths: &serde_json::Map<String, serde_json::Value>,
) -> Result<BTreeSet<(String, String)>> {
    let mut endpoints = BTreeSet::new();
    for (path, path_item) in paths {
        if IGNORED_PATHS.contains(&path.as_str()) {
            continue;
        }
        let path_item = path_item
            .as_object()
            .ok_or_else(|| anyhow!("openapi path '{}' is not an object", path))?;
        for method in path_item.keys() {
            if is_http_method(method) {
                endpoints.insert((method.to_ascii_lowercase(), path.clone()));
            }
        }
    }
    Ok(endpoints)
}

fn mapped_endpoints() -> (BTreeSet<(String, String)>, Vec<String>) {
    let known_case_ids: HashSet<&'static str> = crate::compat::all_cases()
        .iter()
        .map(|case| case.id.0)
        .collect();

    let mut mapped = BTreeSet::new();
    let mut missing_case_ids = Vec::new();

    for (method, path, case_id) in ENDPOINT_CASE_MAP {
        mapped.insert(((*method).to_string(), (*path).to_string()));
        if !known_case_ids.contains(case_id) {
            missing_case_ids.push(format!("- {} {} -> {}", method, path, case_id));
        }
    }

    (mapped, missing_case_ids)
}

fn diff_endpoints(
    left: &BTreeSet<(String, String)>,
    right: &BTreeSet<(String, String)>,
) -> Vec<String> {
    left.difference(right)
        .map(|(method, path)| format!("- {} {}", method, path))
        .collect()
}

fn is_http_method(method: &str) -> bool {
    matches!(
        method,
        "get" | "post" | "put" | "patch" | "delete" | "options" | "head"
    )
}
