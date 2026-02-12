use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bankai_types::api::blocks::{
    BlockDetailDto, BlockStatusDto, BlockSummaryDto, LatestBlockQueryDto,
};
use bankai_types::api::proofs::{BankaiBlockProofDto, BlockProofPayloadDto, ProofFormatDto};
use bankai_types::api::stats::PageDto;
use cairo_air::CairoProof;
use cairo_air::utils::{deserialize_proof_from_file, ProofFormat};
use serde::Serialize;
use starknet_ff::FieldElement;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_serialize::deserialize::CairoDeserialize;

use super::{handle_response, ApiCore};
use crate::errors::{SdkError, SdkResult};

#[derive(Debug, Default, Serialize)]
pub struct BlocksQuery {
    pub status: Option<BlockStatusDto>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Default, Serialize)]
pub struct BlockProofQuery {
    pub block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_format: Option<ProofFormatDto>,
}

pub struct BlocksApi {
    core: Arc<ApiCore>,
}

impl BlocksApi {
    pub(crate) fn new(core: Arc<ApiCore>) -> Self {
        Self { core }
    }

    /// List blocks with optional pagination and status filter.
    pub async fn list(&self, query: &BlocksQuery) -> SdkResult<PageDto<BlockSummaryDto>> {
        let url = format!("{}/v1/blocks", self.core.base_url);
        let response = self.core.client.get(&url).query(query).send().await?;
        handle_response(response).await
    }

    /// Fetches the latest block summary with optional status filter.
    pub async fn latest(&self, query: &LatestBlockQueryDto) -> SdkResult<BlockSummaryDto> {
        let url = format!("{}/v1/blocks/latest", self.core.base_url);
        let response = self.core.client.get(&url).query(query).send().await?;
        handle_response(response).await
    }

    /// Fetches the latest completed block number.
    pub async fn latest_number(&self) -> SdkResult<u64> {
        let query = LatestBlockQueryDto {
            status: Some(BlockStatusDto::Completed),
        };
        let block_summary = self.latest(&query).await?;
        Ok(block_summary.height)
    }

    /// Fetches a block by height.
    pub async fn by_height(&self, height: u64) -> SdkResult<BlockDetailDto> {
        let url = format!("{}/v1/blocks/{}", self.core.base_url, height);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }

    /// Fetches the STWO block proof for a specific height (alias endpoint).
    pub async fn proof(&self, height: u64) -> SdkResult<BankaiBlockProofDto> {
        let url = format!("{}/v1/blocks/{}/proof", self.core.base_url, height);
        let response = self.core.client.get(&url).send().await?;
        handle_response(response).await
    }

    /// Fetches the STWO block proof for a specific height with explicit payload format.
    pub async fn proof_with_format(
        &self,
        height: u64,
        proof_format: ProofFormatDto,
    ) -> SdkResult<BankaiBlockProofDto> {
        let query = BlockProofQuery {
            block_number: Some(height),
            proof_format: Some(proof_format),
        };
        self.proof_by_query(&query).await
    }

    /// Fetches the STWO block proof via the query endpoint.
    pub async fn proof_by_query(&self, query: &BlockProofQuery) -> SdkResult<BankaiBlockProofDto> {
        let url = format!("{}/v1/blocks/get_proof", self.core.base_url);
        let response = self.core.client.get(&url).query(query).send().await?;
        handle_response(response).await
    }
}

pub(crate) fn parse_block_proof_payload(
    payload: BlockProofPayloadDto,
) -> SdkResult<CairoProof<Blake2sMerkleHasher>> {
    match payload {
        BlockProofPayloadDto::Bin(value) => parse_binary_block_proof_payload(&value),
        BlockProofPayloadDto::Json(value) => parse_json_block_proof_payload(value),
    }
}

fn parse_json_block_proof_payload(
    value: serde_json::Value,
) -> SdkResult<CairoProof<Blake2sMerkleHasher>> {
    if let Ok(parsed) = serde_json::from_value::<CairoProof<Blake2sMerkleHasher>>(value.clone()) {
        Ok(parsed)
    } else {
        let felt_strings: Vec<String> = serde_json::from_value(value).map_err(|e| {
            SdkError::InvalidInput(format!(
                "json block proof must be either CairoProof object or cairo-serde felt array: {e}"
            ))
        })?;

        let data: Vec<FieldElement> = felt_strings
            .iter()
            .map(|v| {
                v.parse().map_err(|e| {
                    SdkError::InvalidInput(format!(
                        "failed to parse cairo-serde field element '{v}': {e}"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let res = CairoProof::<Blake2sMerkleHasher>::deserialize(&mut data.iter());
        Ok(res)
    }
}

fn parse_binary_block_proof_payload(
    value: &str,
) -> SdkResult<CairoProof<Blake2sMerkleHasher>> {
    let proof_path = build_temp_binary_proof_path()?;

    std::fs::write(&proof_path, value.as_bytes()).map_err(|e| {
        SdkError::Other(format!(
            "failed to write temporary binary block proof '{}': {e}",
            proof_path.display()
        ))
    })?;

    let parsed = deserialize_proof_from_file::<Blake2sMerkleHasher>(&proof_path, ProofFormat::Binary)
        .map_err(|e| SdkError::InvalidInput(format!("failed to deserialize binary block proof: {e}")));

    let _ = std::fs::remove_file(&proof_path);
    parsed
}

fn build_temp_binary_proof_path() -> SdkResult<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| SdkError::Other(format!("failed to generate temp proof path: {e}")))?;
    let mut path = std::env::temp_dir();
    path.push(format!(
        "bankai-sdk-proof-{}-{}.bin",
        std::process::id(),
        ts.as_nanos()
    ));
    Ok(path)
}
