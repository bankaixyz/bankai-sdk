use crate::{proofs::MmrProofDto, verify::evm::beacon::BeaconHeader};

#[cfg_attr(any(feature = "verifier-types", feature = "std"), derive(Debug, Clone))]
pub struct BeaconHeaderProof {
    pub header: BeaconHeader,
    pub mmr_proof: MmrProofDto,
}
