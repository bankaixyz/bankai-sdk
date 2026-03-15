#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompatCaseId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatArea {
    Health,
    Chains,
    Blocks,
    EthereumBeacon,
    EthereumExecution,
    EthereumRoot,
    OpStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixScope {
    Core,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdkCallSpec {
    HealthGet,
    ChainsList,
    ChainsByIdFromList,
    ChainsSummaryByIdFromList,
    ExplorerOverview,
    BlocksList,
    BlocksLatestCompleted,
    BlocksByHeightFromLatest,
    BlocksFullByHeightFromLatest,
    BlocksProofByQueryFromLatest,
    BlocksProofByHeightFromLatest,
    BlocksMmrProofFromLatest,
    BlocksBlockProofFromLatest,
    EthereumEpochFinalized,
    EthereumEpochByNumberFromEpoch,
    EthereumSyncCommitteeFromEpoch,
    EthereumBeaconHeightFinalized,
    EthereumBeaconSnapshotFinalized,
    EthereumBeaconMmrRootFinalized,
    EthereumBeaconMmrProofFromSnapshot,
    EthereumBeaconLightClientProofFromSnapshot,
    EthereumExecutionHeightFinalized,
    EthereumExecutionSnapshotFinalized,
    EthereumExecutionMmrRootFinalized,
    EthereumExecutionMmrProofFromSnapshot,
    EthereumExecutionLightClientProofFromSnapshot,
    OpStackHeightFinalized,
    OpStackSnapshotFinalized,
    OpStackMerkleProofFromSnapshot,
    OpStackMmrProofFromSnapshot,
    OpStackLightClientProofFromSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmrProofSource {
    EthereumBeacon,
    EthereumExecution,
    OpStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankaiMmrProofSource {
    BlocksMmrProofEndpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightClientProofSource {
    EthereumBeacon,
    EthereumExecution,
    OpStack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerkleProofSource {
    OpStackFromSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofHashSource {
    BlocksBlockProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiErrorSource {
    SyncCommitteeFromEpoch,
    FilterConflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatKind {
    SdkCallDecode {
        call: SdkCallSpec,
        scope: MatrixScope,
    },
    ProofHashConsistency {
        source: ProofHashSource,
        scope: MatrixScope,
    },
    MmrProofVerify {
        source: MmrProofSource,
        scope: MatrixScope,
    },
    MerkleProofVerify {
        source: MerkleProofSource,
        scope: MatrixScope,
    },
    BankaiMmrProofVerify {
        source: BankaiMmrProofSource,
        scope: MatrixScope,
    },
    LightClientProofVerify {
        source: LightClientProofSource,
        scope: MatrixScope,
    },
    ApiErrorShape {
        source: ApiErrorSource,
        scope: MatrixScope,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatEndpoint {
    pub method: HttpMethod,
    pub path: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatCaseDef {
    pub id: CompatCaseId,
    pub area: CompatArea,
    pub kind: CompatKind,
    pub endpoint: Option<CompatEndpoint>,
    pub required: bool,
}
