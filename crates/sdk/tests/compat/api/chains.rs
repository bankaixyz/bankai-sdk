use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind, HttpMethod, MatrixScope,
    SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("chains.list.decode"),
            area: CompatArea::Chains,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::ChainsList,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/chains",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("chains.by_id.decode"),
            area: CompatArea::Chains,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::ChainsByIdFromList,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/chains/{chain_id}",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("chains.summary.decode"),
            area: CompatArea::Chains,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::ChainsSummaryByIdFromList,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/chains/{chain_id}/summary",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("explorer.overview.decode"),
            area: CompatArea::Chains,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::ExplorerOverview,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/explorer/overview",
            }),
            required: true,
        },
    ]
}
