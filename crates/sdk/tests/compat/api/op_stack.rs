use crate::compat::case::{
    CompatArea, CompatCaseDef, CompatCaseId, CompatEndpoint, CompatKind, HttpMethod,
    LightClientProofSource, MatrixScope, MerkleProofSource, MmrProofSource, SdkCallSpec,
};

pub fn cases() -> Vec<CompatCaseDef> {
    vec![
        CompatCaseDef {
            id: CompatCaseId("op_stack.height.decode"),
            area: CompatArea::OpStack,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::OpStackHeightFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/op/{name}/height",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("op_stack.snapshot.decode"),
            area: CompatArea::OpStack,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::OpStackSnapshotFinalized,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Get,
                path: "/v1/op/{name}/snapshot",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("op_stack.merkle_proof.decode"),
            area: CompatArea::OpStack,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::OpStackMerkleProofFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/op/{name}/merkle_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("op_stack.mmr_proof.decode"),
            area: CompatArea::OpStack,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::OpStackMmrProofFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/op/{name}/mmr_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("op_stack.light_client_proof.decode"),
            area: CompatArea::OpStack,
            kind: CompatKind::SdkCallDecode {
                call: SdkCallSpec::OpStackLightClientProofFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: Some(CompatEndpoint {
                method: HttpMethod::Post,
                path: "/v1/op/{name}/light_client_proof",
            }),
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("op_stack.merkle_proof.verify"),
            area: CompatArea::OpStack,
            kind: CompatKind::MerkleProofVerify {
                source: MerkleProofSource::OpStackFromSnapshot,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("op_stack.mmr_proof.verify"),
            area: CompatArea::OpStack,
            kind: CompatKind::MmrProofVerify {
                source: MmrProofSource::OpStack,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
        CompatCaseDef {
            id: CompatCaseId("op_stack.light_client_proof.verify"),
            area: CompatArea::OpStack,
            kind: CompatKind::LightClientProofVerify {
                source: LightClientProofSource::OpStack,
                scope: MatrixScope::Core,
            },
            endpoint: None,
            required: true,
        },
    ]
}
