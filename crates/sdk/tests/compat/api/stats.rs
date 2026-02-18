use crate::compat::case::{CompatArea, CompatCaseDef, CompatCaseId, CompatKind, SdkCallSpec};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("stats.overview.decode"),
            area: CompatArea::Stats,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::StatsOverview,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("stats.block_detail.decode"),
            area: CompatArea::Stats,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::StatsBlockDetailFromLatest,
            },
            required: true,
        },
    ]
}
