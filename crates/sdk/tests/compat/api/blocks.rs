use crate::compat::case::{
    BankaiMmrProofSource, CompatArea, CompatCaseDef, CompatCaseId, CompatKind, ProofHashSource,
    SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("blocks.list.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksList,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.latest.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksLatestCompleted,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.by_height.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksByHeightFromLatest,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.proof_by_query.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksProofByQueryFromLatest,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.proof_by_height.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksProofByHeightFromLatest,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.mmr_proof.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksMmrProofFromLatest,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.block_proof.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksBlockProofFromLatest,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.block_proof.hash_consistency"),
            area: CompatArea::Blocks,
            kind: CompatKind::ProofHashConsistency {
                source: ProofHashSource::BlocksBlockProof,
            },
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.mmr_proof.verify"),
            area: CompatArea::Blocks,
            kind: CompatKind::BankaiMmrProofVerify {
                source: BankaiMmrProofSource::BlocksMmrProofEndpoint,
            },
            required: true,
        },
    ]
}
