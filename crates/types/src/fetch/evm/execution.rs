use crate::api::MmrProofDto;
use alloy_rpc_types::Header as ExecutionHeader;
use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;

pub struct ExecutionHeaderProof {
    pub header: ExecutionHeader,
    pub block_proof: CairoProof<Blake2sMerkleHasher>,
    pub mmr_proof: MmrProofDto,
}
