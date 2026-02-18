use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatKind, LightClientProofSource, MmrProofSource,
    SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.height.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionHeightFinalized,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.snapshot.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionSnapshotFinalized,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.mmr_root.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionMmrRootFinalized,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.mmr_proof.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionMmrProofFromSnapshot,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.light_client_proof.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionLightClientProofFromSnapshot,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.mmr_proof.verify"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::MmrProofVerify {
                source: MmrProofSource::EthereumExecutionFromSnapshot,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.light_client_proof.verify"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::LightClientProofVerify {
                source: LightClientProofSource::EthereumExecutionFromSnapshot,
            },
            required: true,
        },
    ]
}
