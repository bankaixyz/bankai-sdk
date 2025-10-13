use crate::{api::proofs::MmrProofDto, verify::evm::beacon::BeaconHeader};

#[derive(Debug)]
pub struct BeaconHeaderProof {
    pub header: BeaconHeader,
    pub mmr_proof: MmrProofDto,
}