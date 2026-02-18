use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind, HttpMethod,
    LightClientProofSource, MatrixScope, MmrProofSource, SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.height.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionHeightFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/execution/height",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.snapshot.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionSnapshotFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/execution/snapshot",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.mmr_root.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionMmrRootFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/execution/mmr_root",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.mmr_proof.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionMmrProofFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/ethereum/execution/mmr_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.light_client_proof.decode"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumExecutionLightClientProofFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/ethereum/execution/light_client_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.mmr_proof.verify"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::MmrProofVerify {
                source: MmrProofSource::EthereumExecutionFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.execution.light_client_proof.verify"),
            area: CompatArea::EthereumExecution,
            kind: CompatKind::LightClientProofVerify {
                source: LightClientProofSource::EthereumExecutionFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
    ]
}
