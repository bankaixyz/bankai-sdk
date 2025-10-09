use alloy_rpc_types::Header as ExecutionHeader;
use cairo_air::CairoProof;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use crate::api::MmrProofDto;

pub struct HeaderProof {
    pub header: ExecutionHeader,
    pub block_proof: CairoProof<Blake2sMerkleHasher>,
    pub mmr_proof: MmrProofDto,
}