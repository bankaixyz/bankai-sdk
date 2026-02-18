use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind, HttpMethod, MatrixScope,
    SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("stats.overview.decode"),
            area: CompatArea::Stats,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::StatsOverview,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/stats/overview",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("stats.block_detail.decode"),
            area: CompatArea::Stats,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::StatsBlockDetailFromLatest,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/stats/block/{height}",
            }),
            required: true,
        },
    ]
}
