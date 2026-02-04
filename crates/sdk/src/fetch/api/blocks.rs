use std::sync::Arc;

use bankai_types::api::blocks::{
    BlockDetailDto, BlockStatusDto, BlockSummaryDto, LatestBlockQueryDto,
};
use bankai_types::api::proofs::BankaiBlockProofDto;
use bankai_types::api::stats::PageDto;
use cairo_air::CairoProof;
use serde::Serialize;
use starknet_ff::FieldElement;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_serialize::deserialize::CairoDeserialize;

use crate::errors::SdkResult;
use super::{handle_response, ApiCore};

#[derive(Debug, Default, Serialize)]
pub struct BlocksQuery {
    pub status: Option<BlockStatusDto>,
    pub cursor: Option<String>,
    pub limit: Option<u64>,
}

#[derive(Debug, Default, Serialize)]
pub struct BlockProofQuery {
    pub block_number: Option<u64>,
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

    /// Fetches the latest block summary (request body required).
    pub async fn latest(&self, query: &LatestBlockQueryDto) -> SdkResult<BlockSummaryDto> {
        let url = format!("{}/v1/blocks/latest", self.core.base_url);
        let response = self.core.client.get(&url).json(query).send().await?;
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

    /// Fetches the STWO block proof via the query endpoint.
    pub async fn proof_by_query(&self, query: &BlockProofQuery) -> SdkResult<BankaiBlockProofDto> {
        let url = format!("{}/v1/blocks/get_proof", self.core.base_url);
        let response = self.core.client.get(&url).query(query).send().await?;
        handle_response(response).await
    }
}

pub(crate) fn parse_block_proof_value(
    value: serde_json::Value,
) -> SdkResult<CairoProof<Blake2sMerkleHasher>> {
    if let Ok(parsed) = serde_json::from_value::<CairoProof<Blake2sMerkleHasher>>(value.clone()) {
        Ok(parsed)
    } else {
        let data: Vec<FieldElement> = value
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().parse().unwrap())
            .collect();
        let res = CairoProof::<Blake2sMerkleHasher>::deserialize(&mut data.iter());
        Ok(res)
    }
}
