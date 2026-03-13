use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind, HttpMethod, MatrixScope,
    SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![CompatCaseDef {
        id: CompatCaseId("health.get.decode"),
        area: CompatArea::Health,
        kind: CompatKind::SdkCallDecode {
            call: SdkCallSpec::HealthGet,
            scope: MatrixScope::Core,
        },
        endpoint: Some(CompatEndpoint {
            method: HttpMethod::Get,
            path: "/v1/health",
        }),
        required: true,
    }]
}
