use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::api::blocks::{EthereumConsensusSummaryDto, MmrRootsDto};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockDetailStatsDto {
    pub height: u64,
    pub e2e_ms: Option<u64>,
    pub updated_at: String,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beacon_mmr: Option<ChainMmrInfoDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mmr: Option<ChainMmrInfoDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainOverviewDto {
    pub network_id: u64,
    pub name: String,
    pub total_headers: u64,
    pub first_header: u64,
    pub current_header: u64,
    pub latest_justified_height: u64,
    pub latest_finalized_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LatestBlockOverviewDto {
    pub height: u64,
    pub e2e_ms: Option<u64>,
    pub updated_at: String,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OverviewStatsDto {
    pub totals: TotalsDto,
    pub chains: Vec<ChainOverviewDto>,
    pub latest: PageDto<LatestBlockOverviewDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotalsDto {
    pub total_blocks: u64,
    pub total_proofs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_e2e_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(bound = "T: ToSchema")]
pub struct PageDto<T> {
    pub data: Vec<T>,
    pub meta: PageMetaDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainMmrInfoDto {
    pub keccak_root: String,
    pub poseidon_root: String,
    pub elements_count: u64,
    pub leafs_count: u64,
    pub keccak_peaks_count: u64,
    pub poseidon_peaks_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keccak_peaks: Option<Vec<String>>, // only when detail=true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poseidon_peaks: Option<Vec<String>>, // only when detail=true
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainSnapshotSummaryDto {
    pub chain_id: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub justified_height: u64,
    pub finalized_height: u64,
    pub mmr_roots: MmrRootsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageMetaDto {
    pub cursor: Option<String>,
    pub has_more: bool,
}