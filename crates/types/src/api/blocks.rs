use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::{
    ethereum::BankaiBlockFilterDto, op_stack::OpChainsSummaryDto, stats::ChainSnapshotSummaryDto,
};
use crate::common::{HashingFunction, ProofFormat};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(bound = "T: ToSchema")]
pub struct BlockWithHashDto<T> {
    pub block_hash: String,
    pub block: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockDetailDto {
    pub height: u64,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub program_hash: String,
    pub status: BlockStatusDto,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
    #[serde(default)]
    pub op_chains: Option<OpChainsSummaryDto>,
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
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub program_hash: String,
    pub status: BlockStatusDto,
    pub ethereum: Option<EthereumConsensusSummaryDto>,
    #[serde(default)]
    pub op_chains: Option<OpChainsSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EthereumConsensusSummaryDto {
    pub epoch_number: u64,
    pub epochs_count: u32,
    pub num_signers: u64,
    pub beacon: Option<ChainSnapshotSummaryDto>,
    pub execution: Option<ChainSnapshotSummaryDto>,
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BankaiTargetBlockSelectorDto {
    pub block_number: Option<u64>,
    pub block_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BankaiMmrProofRequestDto {
    pub filter: BankaiBlockFilterDto,
    pub target_block: BankaiTargetBlockSelectorDto,
    pub hashing_function: HashingFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BankaiBlockProofRequestDto {
    pub filter: BankaiBlockFilterDto,
    pub target_block: BankaiTargetBlockSelectorDto,
    pub hashing_function: HashingFunction,
    #[serde(default)]
    pub proof_format: ProofFormat,
}

/// API envelope carrying canonical Bankai block hash + full block payload.
pub type BankaiBlockOutputDto = crate::block::BankaiBlockOutput;

/// API envelope carrying canonical Bankai block hash + full block payload with OP chain clients.
pub type BankaiBlockFullOutputDto = crate::block::BankaiBlockFullOutput;
