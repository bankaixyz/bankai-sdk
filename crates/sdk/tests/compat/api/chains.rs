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
    ]
}
