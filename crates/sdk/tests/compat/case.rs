#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompatCaseId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatArea {
    Health,
    Chains,
    Blocks,
    Stats,
    EthereumBeacon,
    EthereumExecution,
    EthereumRoot,
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
    BlocksList,
    BlocksLatestCompleted,
    BlocksByHeightFromLatest,
    BlocksProofByQueryFromLatest,
    BlocksProofByHeightFromLatest,
    BlocksMmrProofFromLatest,
    BlocksBlockProofFromLatest,
    StatsOverview,
    StatsBlockDetailFromLatest,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmrProofSource {
    EthereumBeaconFromSnapshot,
    EthereumExecutionFromSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BankaiMmrProofSource {
    BlocksMmrProofEndpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightClientProofSource {
    EthereumBeaconFromSnapshot,
    EthereumExecutionFromSnapshot,
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
