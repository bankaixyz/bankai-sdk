use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockDetailDto {
    pub height: u64,
    pub status: BlockStatusDto,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
    pub zk_proof_available: bool,
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
pub struct BlockSummaryDto {
    pub height: u64,
    pub status: BlockStatusDto,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
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
pub struct ChainSnapshotSummaryDto {
    pub chain_id: u64,
    pub start_height: u64,
    pub end_height: u64,
    pub justified_height: u64,
    pub finalized_height: u64,
    pub mmr_roots: MmrRootsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MmrRootsDto {
    pub keccak_root: String,   // 0x…32
    pub poseidon_root: String, // 0x…32
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LatestBlockQueryDto {
    pub status: Option<BlockStatusDto>,
}