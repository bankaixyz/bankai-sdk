use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatKind, LightClientProofSource, MmrProofSource,
    SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.height.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconHeightFinalized,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.snapshot.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconSnapshotFinalized,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.mmr_root.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconMmrRootFinalized,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.mmr_proof.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconMmrProofFromSnapshot,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.light_client_proof.decode"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumBeaconLightClientProofFromSnapshot,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.mmr_proof.verify"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::MmrProofVerify {
                source: MmrProofSource::EthereumBeaconFromSnapshot,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.beacon.light_client_proof.verify"),
            area: CompatArea::EthereumBeacon,
            kind: CompatKind::LightClientProofVerify {
                source: LightClientProofSource::EthereumBeaconFromSnapshot,
            },
            required: true,
        },
    ]
}
