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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeAs {
    JsonValue,
    BankaiBlockProofWithMmr,
    BankaiBlockProofWithBlock,
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
pub enum RawBodySource {
    BankaiMmrProofRequestFromLatest,
    BankaiBlockProofRequestFromLatest,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatKind {
    SdkCallDecode {
        call: SdkCallSpec,
    },
    #[allow(dead_code)]
    RawHttpDecode {
        method: HttpMethod,
        path: &'static str,
        query: &'static [(&'static str, &'static str)],
        body: Option<RawBodySource>,
        decode_as: DecodeAs,
    },
    ProofHashConsistency {
        source: ProofHashSource,
    },
    MmrProofVerify {
        source: MmrProofSource,
    },
    BankaiMmrProofVerify {
        source: BankaiMmrProofSource,
    },
    LightClientProofVerify {
        source: LightClientProofSource,
    },
    ApiErrorShape {
        source: ApiErrorSource,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatCaseDef {
    pub id: CompatCaseId,
    pub area: CompatArea,
    pub kind: CompatKind,
    pub required: bool,
}
