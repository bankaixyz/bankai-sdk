use crate::compat::case::{CompatArea, CompatCaseDef, CompatCaseId, CompatKind, SdkCallSpec};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![CompatCaseDef {
        id: CompatCaseId("health.get.decode"),
        area: CompatArea::Health,
        kind: CompatKind::SdkCallDecode {
            call: SdkCallSpec::HealthGet,
        },
        required: true,
    }]
}
