use std::collections::BTreeSet;

use anyhow::{anyhow, Context, Result};

use crate::compat::context::CompatContext;

const IGNORED_PATHS: &[&str] = &["/v1/openapi.json", "/v1/swagger", "/v1/rapidoc"];

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
    let mapped = mapped_endpoints();

    if mapped.is_empty() {
        return Err(anyhow!(
            "SDK endpoint coverage map from compat case metadata is empty"
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

fn mapped_endpoints() -> BTreeSet<(String, String)> {
    let mut mapped = BTreeSet::new();

    for case in crate::compat::all_cases() {
        if let Some(endpoint) = case.endpoint {
            let method = match endpoint.method {
                crate::compat::case::HttpMethod::Get => "get",
                crate::compat::case::HttpMethod::Post => "post",
            };
            mapped.insert((method.to_string(), endpoint.path.to_string()));
        }
    }

    mapped
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
