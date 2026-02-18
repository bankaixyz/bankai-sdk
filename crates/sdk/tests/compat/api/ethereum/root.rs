use crate::compat::case::{
    ApiErrorSource, CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind,
    HttpMethod, MatrixScope, SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("ethereum.epoch.decode"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumEpochFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/epoch",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.epoch_by_number.decode"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumEpochByNumberFromEpoch,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/epoch/{number}",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.sync_committee.decode"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::EthereumSyncCommitteeFromEpoch,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/sync_committee",
            }),
            required: false,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.sync_committee.error_shape"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::ApiErrorShape {
                source: ApiErrorSource::SyncCommitteeFromEpoch,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/sync_committee",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("ethereum.filter_conflict.error_shape"),
            area: CompatArea::EthereumRoot,
            kind: CompatKind::ApiErrorShape {
                source: ApiErrorSource::FilterConflict,
                scope: MatrixScope::Edge,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/ethereum/epoch",
            }),
            required: false,
        },
    ]
}
