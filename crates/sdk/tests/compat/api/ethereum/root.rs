use crate::compat::case::{
    ApiErrorSource, CompatArea, CompatCaseDef, CompatCaseId, CompatKind, SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("ethereum.epoch.decode"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumEpochFinalized,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.epoch_by_number.decode"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumEpochByNumberFromEpoch,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.sync_committee.decode"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumSyncCommitteeFromEpoch,
            },
            required: false,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.sync_committee.error_shape"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::ApiErrorShape {
                source: ApiErrorSource::SyncCommitteeFromEpoch,
            },
            required: true,
        },
    ]
}
