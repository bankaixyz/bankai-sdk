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
    pub total_headers_tracked: Option<u64>,
    pub first_tracked_height: Option<u64>,
    pub mmr_meta: Option<MmrMetaDto>,
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrMetaDto {
    pub elements_count: u64,
    pub leafs_count: u64,
    pub keccak_peaks_count: u64,
    pub poseidon_peaks_count: u64,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::ChainSummaryDto;

    #[test]
    fn chain_summary_decodes_new_optional_fields() {
        let payload = json!({
            "chain": {
                "integration_id": 1,
                "chain_id": 11155111,
                "name": "sepolia",
                "ecosystem": "ethereum",
                "chain_type": "execution_layer",
                "active": true,
                "parent_chain_id": null,
                "activation_block_height": null
            },
            "state": {
                "status": "active",
                "activation_block_height": null
            },
            "latest_snapshot": null,
            "latest_bankai_block_height": null,
            "total_headers_tracked": 24767,
            "first_tracked_height": 38671974,
            "mmr_meta": {
                "elements_count": 49519,
                "leafs_count": 24767,
                "keccak_peaks_count": 14,
                "poseidon_peaks_count": 14
            }
        });

        let summary: ChainSummaryDto = serde_json::from_value(payload).expect("summary json");
        assert_eq!(summary.total_headers_tracked, Some(24_767));
        assert_eq!(summary.first_tracked_height, Some(38_671_974));
        let mmr_meta = summary.mmr_meta.expect("mmr meta");
        assert_eq!(mmr_meta.elements_count, 49_519);
        assert_eq!(mmr_meta.leafs_count, 24_767);
        assert_eq!(mmr_meta.keccak_peaks_count, 14);
        assert_eq!(mmr_meta.poseidon_peaks_count, 14);
    }
}
