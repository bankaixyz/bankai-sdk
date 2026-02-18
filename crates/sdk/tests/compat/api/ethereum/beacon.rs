use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind, HttpMethod,
    LightClientProofSource, MatrixScope, MmrProofSource, SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.height.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconHeightFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/beacon/height",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.snapshot.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconSnapshotFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/beacon/snapshot",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.mmr_root.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconMmrRootFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/beacon/mmr_root",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.mmr_proof.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconMmrProofFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/ethereum/beacon/mmr_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.light_client_proof.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconLightClientProofFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/ethereum/beacon/light_client_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.mmr_proof.verify"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::MmrProofVerify {
                source: MmrProofSource::EthereumBeaconFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.light_client_proof.verify"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::LightClientProofVerify {
                source: LightClientProofSource::EthereumBeaconFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
    ]
}
