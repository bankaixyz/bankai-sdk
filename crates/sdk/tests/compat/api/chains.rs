use crate::compat::case::{CompatArea, CompatCaseDef, CompatCaseId, CompatKind, SdkCallSpec};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("chains.list.decode"),
            area: CompatArea::Chains,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::ChainsList,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("chains.by_id.decode"),
            area: CompatArea::Chains,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::ChainsByIdFromList,
            },
            required: true,
        },
    ]
}
