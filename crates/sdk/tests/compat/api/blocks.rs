use crate::compat::case::{
    BankaiMmrProofSource, CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind,
    HttpMethod, MatrixScope, ProofHashSource, SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("blocks.list.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksList,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/blocks",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.latest.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksLatestCompleted,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/blocks/latest",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.by_height.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksByHeightFromLatest,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/blocks/{height}",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.proof_by_query.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksProofByQueryFromLatest,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/blocks/get_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.proof_by_height.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksProofByHeightFromLatest,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/blocks/{height}/proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.mmr_proof.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksMmrProofFromLatest,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/blocks/mmr_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.block_proof.decode"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksBlockProofFromLatest,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/blocks/block_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.mmr_proof.decode.edge"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksMmrProofFromLatest,
                scope: MatrixScope::Edge,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/blocks/mmr_proof",
            }),
            required: false,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.block_proof.decode.edge"),
            area: CompatArea::Blocks,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::BlocksBlockProofFromLatest,
                scope: MatrixScope::Edge,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/blocks/block_proof",
            }),
            required: false,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.block_proof.hash_consistency"),
            area: CompatArea::Blocks,
            kind: CompatKind::ProofHashConsistency {
                source: ProofHashSource::BlocksBlockProof,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.block_proof.hash_consistency.edge"),
            area: CompatArea::Blocks,
            kind: CompatKind::ProofHashConsistency {
                source: ProofHashSource::BlocksBlockProof,
                scope: MatrixScope::Edge,
            },
            endpoint: None,
            required: false,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.mmr_proof.verify"),
            area: CompatArea::Blocks,
            kind: CompatKind::BankaiMmrProofVerify {
                source: BankaiMmrProofSource::BlocksMmrProofEndpoint,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("blocks.mmr_proof.verify.edge"),
            area: CompatArea::Blocks,
            kind: CompatKind::BankaiMmrProofVerify {
                source: BankaiMmrProofSource::BlocksMmrProofEndpoint,
                scope: MatrixScope::Edge,
            },
            endpoint: None,
            required: false,
        },
    ]
}
