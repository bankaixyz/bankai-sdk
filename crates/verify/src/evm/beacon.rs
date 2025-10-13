extern crate alloc;
use alloc::format;
use alloc::string::String;

use bankai_types::fetch::evm::beacon::BeaconHeaderProof;
use bankai_types::verify::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use alloy_primitives::hex::ToHexExt;

use crate::bankai::mmr::BankaiMmr;
use crate::VerifyError;

pub struct BeaconVerifier;

impl BeaconVerifier {
    pub fn verify_header_proof(
        proof: &BeaconHeaderProof,
        root: String,
    ) -> Result<BeaconHeader, VerifyError> {
        if proof.mmr_proof.root != root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        BankaiMmr::verify_mmr_proof(proof.mmr_proof.clone())?;

        let hash = proof.header.tree_hash_root();
        let expected_header_hash = format!("0x{}", hash.encode_hex());
        if expected_header_hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone())
    }
}
