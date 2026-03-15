use crate::compat::case::{
    ApiErrorSource, CompatCaseDef, CompatKind, HttpMethod, LightClientProofSource, MatrixScope,
    MerkleProofSource, MmrProofSource, ProofHashSource, SdkCallSpec,
};
use crate::compat::context::CompatContext;

pub(super) async fn debug_curl_for_case(ctx: &CompatContext, case: &CompatCaseDef) -> String {
    match case.kind {
        CompatKind::SdkCallDecode { call, scope } => {
            debug_curl_for_sdk_call(ctx, call, scope).await
        }
        CompatKind::ProofHashConsistency {
            source: ProofHashSource::BlocksBlockProof,
            scope,
        } => {
            let block = debug_blocks_body_call(ctx, "/v1/blocks/block_proof", scope).await;
            let mmr = debug_blocks_body_call(ctx, "/v1/blocks/mmr_proof", scope).await;
            format!("{block}\n{mmr}")
        }
        CompatKind::MmrProofVerify { source, scope } => {
            debug_curl_for_mmr_verify(ctx, source, scope).await
        }
        CompatKind::MerkleProofVerify { source, scope } => {
            debug_curl_for_merkle_verify(ctx, source, scope).await
        }
        CompatKind::BankaiMmrProofVerify { .. } => {
            debug_blocks_body_call(ctx, "/v1/blocks/mmr_proof", MatrixScope::Core).await
        }
        CompatKind::LightClientProofVerify { source, scope } => {
            debug_curl_for_light_client_verify(ctx, source, scope).await
        }
        CompatKind::ApiErrorShape { source, scope } => {
            debug_curl_for_api_error_shape(ctx, source, scope).await
        }
    }
}

async fn debug_curl_for_sdk_call(
    ctx: &CompatContext,
    call: SdkCallSpec,
    scope: MatrixScope,
) -> String {
    match call {
        SdkCallSpec::HealthGet => build_curl_command(HttpMethod::Get, &ctx.url("/v1/health"), None),
        SdkCallSpec::ChainsList => build_curl_command(HttpMethod::Get, &ctx.url("/v1/chains"), None),
        SdkCallSpec::ChainsByIdFromList => format!(
            "{}\n{}",
            build_curl_command(HttpMethod::Get, &ctx.url("/v1/chains"), None),
            "curl -sS 'http://<api>/v1/chains/<chain_id>'"
        ),
        SdkCallSpec::ChainsSummaryByIdFromList => format!(
            "{}\n{}",
            build_curl_command(HttpMethod::Get, &ctx.url("/v1/chains"), None),
            "curl -sS 'http://<api>/v1/chains/<chain_id>/summary'"
        ),
        SdkCallSpec::ExplorerOverview => {
            build_curl_command(HttpMethod::Get, &ctx.url("/v1/explorer/overview"), None)
        }
        SdkCallSpec::BlocksList => build_curl_command(HttpMethod::Get, &ctx.url("/v1/blocks"), None),
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
        SdkCallSpec::BlocksFullByHeightFromLatest => format!(
            "{}\n{}",
            build_curl_command(
                HttpMethod::Get,
                &format_url_with_query(ctx, "/v1/blocks/latest", &[("status", "completed")]),
                None
            ),
            "curl -sS 'http://<api>/v1/blocks/<height>/full'"
        ),
        SdkCallSpec::BlocksProofByQueryFromLatest => {
            "curl -sS 'http://<api>/v1/blocks/get_proof?block_number=<height>&proof_format=<bin|json>'"
                .to_string()
        }
        SdkCallSpec::BlocksProofByHeightFromLatest => {
            "curl -sS 'http://<api>/v1/blocks/<height>/proof'".to_string()
        }
        SdkCallSpec::BlocksMmrProofFromLatest => {
            debug_blocks_body_call(ctx, "/v1/blocks/mmr_proof", scope).await
        }
        SdkCallSpec::BlocksBlockProofFromLatest => {
            debug_blocks_body_call(ctx, "/v1/blocks/block_proof", scope).await
        }
        SdkCallSpec::EthereumEpochFinalized => build_curl_command(
            HttpMethod::Get,
            &format_url_with_query(ctx, "/v1/ethereum/epoch", &[("selector", "finalized")]),
            None,
        ),
        SdkCallSpec::EthereumEpochByNumberFromEpoch => {
            "curl -sS 'http://<api>/v1/ethereum/epoch/<number>'".to_string()
        }
        SdkCallSpec::EthereumSyncCommitteeFromEpoch => {
            "curl -sS 'http://<api>/v1/ethereum/sync_committee?term_id=<term_id>'".to_string()
        }
        SdkCallSpec::EthereumBeaconHeightFinalized => {
            "curl -sS 'http://<api>/v1/ethereum/beacon/height?selector=<latest|justified|finalized>'"
                .to_string()
        }
        SdkCallSpec::EthereumBeaconSnapshotFinalized => {
            "curl -sS 'http://<api>/v1/ethereum/beacon/snapshot?selector=<latest|justified|finalized>'"
                .to_string()
        }
        SdkCallSpec::EthereumBeaconMmrRootFinalized => {
            "curl -sS 'http://<api>/v1/ethereum/beacon/mmr_root?selector=<latest|justified|finalized>'"
                .to_string()
        }
        SdkCallSpec::EthereumBeaconMmrProofFromSnapshot => {
            debug_curl_for_mmr_verify(ctx, MmrProofSource::EthereumBeacon, scope).await
        }
        SdkCallSpec::EthereumBeaconLightClientProofFromSnapshot => {
            debug_curl_for_light_client_verify(
                ctx,
                LightClientProofSource::EthereumBeacon,
                scope,
            )
            .await
        }
        SdkCallSpec::EthereumExecutionHeightFinalized => {
            "curl -sS 'http://<api>/v1/ethereum/execution/height?selector=<latest|justified|finalized>'"
                .to_string()
        }
        SdkCallSpec::EthereumExecutionSnapshotFinalized => {
            "curl -sS 'http://<api>/v1/ethereum/execution/snapshot?selector=<latest|justified|finalized>'"
                .to_string()
        }
        SdkCallSpec::EthereumExecutionMmrRootFinalized => {
            "curl -sS 'http://<api>/v1/ethereum/execution/mmr_root?selector=<latest|justified|finalized>'"
                .to_string()
        }
        SdkCallSpec::EthereumExecutionMmrProofFromSnapshot => {
            debug_curl_for_mmr_verify(ctx, MmrProofSource::EthereumExecution, scope)
                .await
        }
        SdkCallSpec::EthereumExecutionLightClientProofFromSnapshot => {
            debug_curl_for_light_client_verify(
                ctx,
                LightClientProofSource::EthereumExecution,
                scope,
            )
            .await
        }
        SdkCallSpec::OpStackHeightFinalized => {
            "curl -sS 'http://<api>/v1/op/<name>/height?selector=finalized'".to_string()
        }
        SdkCallSpec::OpStackSnapshotFinalized => {
            "curl -sS 'http://<api>/v1/op/<name>/snapshot?selector=finalized'".to_string()
        }
        SdkCallSpec::OpStackMerkleProofFromSnapshot => {
            "curl -sS -X POST 'http://<api>/v1/op/<name>/merkle_proof' -H 'content-type: application/json' --data '{\"filter\":{\"selector\":\"finalized\"}}'".to_string()
        }
        SdkCallSpec::OpStackMmrProofFromSnapshot => {
            "curl -sS -X POST 'http://<api>/v1/op/<name>/mmr_proof' -H 'content-type: application/json' --data '{\"filter\":{\"selector\":\"finalized\"},\"hashing_function\":\"keccak\",\"header_hash\":\"<header_hash>\"}'".to_string()
        }
        SdkCallSpec::OpStackLightClientProofFromSnapshot => {
            "curl -sS -X POST 'http://<api>/v1/op/<name>/light_client_proof' -H 'content-type: application/json' --data '{\"filter\":{\"selector\":\"finalized\"},\"hashing_function\":\"keccak\",\"header_hashes\":[\"<header_hash>\"],\"proof_format\":\"bin\"}'".to_string()
        }
    }
}

async fn debug_blocks_body_call(ctx: &CompatContext, path: &str, scope: MatrixScope) -> String {
    let body = if scope == MatrixScope::Edge {
        serde_json::json!({
            "filter": { "selector": "finalized", "bankai_block_number": "<latest>" },
            "target_block": { "block_number": "<target>", "block_hash": "<target_hash>" },
            "hashing_function": "poseidon"
        })
    } else if let Ok(req) = ctx.bankai_mmr_request_from_latest().await {
        serde_json::to_value(req).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({
            "filter": { "selector": "finalized" },
            "target_block": { "block_number": "<target>" },
            "hashing_function": "keccak"
        })
    };

    build_curl_command(HttpMethod::Post, &ctx.url(path), Some(&body))
}

async fn debug_curl_for_mmr_verify(
    ctx: &CompatContext,
    source: MmrProofSource,
    scope: MatrixScope,
) -> String {
    match source {
        MmrProofSource::EthereumBeacon => {
            let mut req = serde_json::to_value(ctx.beacon_mmr_proof_request().await.ok()).ok();
            if scope == MatrixScope::Edge {
                req = Some(serde_json::json!({
                    "filter": { "selector": "finalized", "bankai_block_number": "<latest>" },
                    "hashing_function": "poseidon",
                    "header_hash": "<beacon_root>"
                }));
            }
            build_curl_command(
                HttpMethod::Post,
                &ctx.url("/v1/ethereum/beacon/mmr_proof"),
                req.as_ref(),
            )
        }
        MmrProofSource::EthereumExecution => {
            let mut req = serde_json::to_value(ctx.execution_mmr_proof_request().await.ok()).ok();
            if scope == MatrixScope::Edge {
                req = Some(serde_json::json!({
                    "filter": { "selector": "finalized", "bankai_block_number": "<latest>" },
                    "hashing_function": "poseidon",
                    "header_hash": "<execution_header_hash>"
                }));
            }
            build_curl_command(
                HttpMethod::Post,
                &ctx.url("/v1/ethereum/execution/mmr_proof"),
                req.as_ref(),
            )
        }
        MmrProofSource::OpStack => {
            let body = if scope == MatrixScope::Edge {
                serde_json::json!({
                    "filter": { "selector": "finalized", "bankai_block_number": "<latest>" },
                    "hashing_function": "poseidon",
                    "header_hash": "<op_header_hash>"
                })
            } else {
                serde_json::json!({
                    "filter": { "selector": "finalized" },
                    "hashing_function": "keccak",
                    "header_hash": "<op_header_hash>"
                })
            };
            build_curl_command(
                HttpMethod::Post,
                &ctx.url("/v1/op/<name>/mmr_proof"),
                Some(&body),
            )
        }
    }
}

async fn debug_curl_for_merkle_verify(
    ctx: &CompatContext,
    source: MerkleProofSource,
    scope: MatrixScope,
) -> String {
    match source {
        MerkleProofSource::OpStackFromSnapshot => {
            let body = if scope == MatrixScope::Edge {
                serde_json::json!({
                    "filter": { "selector": "finalized", "bankai_block_number": "<latest>" }
                })
            } else {
                serde_json::json!({
                    "filter": { "selector": "finalized" }
                })
            };
            build_curl_command(
                HttpMethod::Post,
                &ctx.url("/v1/op/<name>/merkle_proof"),
                Some(&body),
            )
        }
    }
}

async fn debug_curl_for_light_client_verify(
    ctx: &CompatContext,
    source: LightClientProofSource,
    scope: MatrixScope,
) -> String {
    match source {
        LightClientProofSource::EthereumBeacon => {
            let mut req = serde_json::to_value(ctx.beacon_light_client_request().await.ok()).ok();
            if scope == MatrixScope::Edge {
                req = Some(serde_json::json!({
                    "filter": { "selector": "finalized", "bankai_block_number": "<latest>" },
                    "hashing_function": "poseidon",
                    "header_hashes": ["<beacon_root>"],
                    "proof_format": "json"
                }));
            }
            build_curl_command(
                HttpMethod::Post,
                &ctx.url("/v1/ethereum/beacon/light_client_proof"),
                req.as_ref(),
            )
        }
        LightClientProofSource::EthereumExecution => {
            let mut req =
                serde_json::to_value(ctx.execution_light_client_request().await.ok()).ok();
            if scope == MatrixScope::Edge {
                req = Some(serde_json::json!({
                    "filter": { "selector": "finalized", "bankai_block_number": "<latest>" },
                    "hashing_function": "poseidon",
                    "header_hashes": ["<execution_header_hash>"],
                    "proof_format": "json"
                }));
            }
            build_curl_command(
                HttpMethod::Post,
                &ctx.url("/v1/ethereum/execution/light_client_proof"),
                req.as_ref(),
            )
        }
        LightClientProofSource::OpStack => {
            let body = if scope == MatrixScope::Edge {
                serde_json::json!({
                    "filter": { "selector": "finalized", "bankai_block_number": "<latest>" },
                    "hashing_function": "poseidon",
                    "header_hashes": ["<op_header_hash>"],
                    "proof_format": "json"
                })
            } else {
                serde_json::json!({
                    "filter": { "selector": "finalized" },
                    "hashing_function": "keccak",
                    "header_hashes": ["<op_header_hash>"],
                    "proof_format": "bin"
                })
            };
            build_curl_command(
                HttpMethod::Post,
                &ctx.url("/v1/op/<name>/light_client_proof"),
                Some(&body),
            )
        }
    }
}

async fn debug_curl_for_api_error_shape(
    ctx: &CompatContext,
    source: ApiErrorSource,
    _scope: MatrixScope,
) -> String {
    match source {
        ApiErrorSource::SyncCommitteeFromEpoch => {
            "curl -sS 'http://<api>/v1/ethereum/sync_committee?term_id=<term_id>'".to_string()
        }
        ApiErrorSource::FilterConflict => {
            let bn = ctx
                .latest_completed_height()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "<latest>".to_string());
            build_curl_command(
                HttpMethod::Get,
                &format!(
                    "{}?selector=finalized&bankai_block_number={bn}",
                    ctx.url("/v1/ethereum/epoch")
                ),
                None,
            )
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
