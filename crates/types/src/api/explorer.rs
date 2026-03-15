use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::{
    blocks::{BlockSummaryDto, MmrRootsDto},
    chains::ChainInfoDto,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExplorerOverviewDto {
    pub status: ExplorerStatusDto,
    pub core_chains: Vec<ExplorerChainOverviewDto>,
    pub op_chains: ExplorerOpChainsDto,
    pub recent_blocks: Vec<BlockSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExplorerStatusDto {
    pub api_status: String,
    pub latest_completed_block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExplorerOpChainsDto {
    pub active: Vec<ExplorerChainOverviewDto>,
    pub configured: Vec<ExplorerChainOverviewDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExplorerChainOverviewDto {
    pub chain: ChainInfoDto,
    pub current_height: Option<u64>,
    pub first_available_height: Option<u64>,
    pub latest_justified_height: Option<u64>,
    pub latest_finalized_height: Option<u64>,
    pub latest_l1_submission_block: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainSummaryDto {
    pub chain: ChainInfoDto,
    pub state: ChainSummaryStateDto,
    pub latest_snapshot: Option<ChainLatestSnapshotDto>,
    pub latest_bankai_block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainSummaryStateDto {
    pub status: ChainSummaryStatusDto,
    pub activation_block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChainSummaryStatusDto {
    Active,
    Configured,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainLatestSnapshotDto {
    pub current_height: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub justified_height: Option<u64>,
    pub finalized_height: Option<u64>,
    pub header_hash: Option<String>,
    pub l1_submission_block: Option<u64>,
    pub mmr_roots: MmrRootsDto,
}
