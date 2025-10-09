use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChainInfoDto {
    pub id: u64,
    pub network_id: u64,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlockStatusDto {
    Initial,
    FetchedInputs,
    GeneratingTrace,
    ProveTrace,
    ProvingInProgress,
    RetrievingProof,
    TraceGenerated,
    Proven,
    InputConstructionFailed,
    TraceGenerationFailed,
    ProofSubmissionFailed,
    ProvingFailed,
    ProofRetrievalFailed,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrRootsDto {
    pub keccak_root: String,   // 0x…32
    pub poseidon_root: String, // 0x…32
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
pub struct EthereumConsensusSummaryDto {
    pub epoch_number: u64,
    pub epochs_count: u16,
    pub num_signers: u64,
    pub beacon: Option<ChainSnapshotSummaryDto>,
    pub execution: Option<ChainSnapshotSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockSummaryDto {
    pub height: u64,
    pub status: BlockStatusDto,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockDetailDto {
    pub height: u64,
    pub status: BlockStatusDto,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
    pub zk_proof_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HashingFunctionDto {
    Keccak,
    Poseidon,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ZkProofDto {
    pub block_number: u64,
    #[schema(value_type = Object)]
    pub proof: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrProofRequestDto {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String, // 0x…32
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrProofDto {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String,
    pub root: String,
    pub elements_index: u64,
    pub elements_count: u64,
    pub path: Vec<String>,
    pub peaks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofPackRequestDto {
    pub network_id: u64,
    pub block_number: u64,
    pub hashing_function: HashingFunctionDto,
    pub header_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProofPackDto {
    pub zk: ZkProofDto,
    pub mmr: MmrProofDto,
    pub mmr_root_from_snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageMetaDto {
    pub cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(bound = "T: ToSchema")]
pub struct PageDto<T> {
    pub data: Vec<T>,
    pub meta: PageMetaDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardStatsDto {
    pub latest: LatestBlockStatsDto,
    pub totals: TotalsDto,
    pub chains: Vec<ChainDetailedStatsDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LatestBlockStatsDto {
    pub block: BlockSummaryDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e2e_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TotalsDto {
    pub total_blocks: u64,
    pub total_proofs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_e2e_ms: Option<u64>,
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
pub struct ChainDetailedStatsDto {
    pub network_id: u64,
    pub name: String,
    pub first_header: u64,
    pub current_header: u64,
    pub mmr: ChainMmrInfoDto,
}

// New overview and detail DTOs

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
