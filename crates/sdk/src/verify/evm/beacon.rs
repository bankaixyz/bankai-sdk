use alloy_rpc_types_beacon::header::BeaconBlockHeader;
use bankai_types::api::HashingFunctionDto;
use bankai_types::fetch::evm::beacon::{BeaconHeader, BeaconHeaderProof};
use bankai_types::fetch::evm::execution::ExecutionHeaderProof;
use tree_hash::TreeHash;
use alloy_rpc_types::Header as ExecutionHeader;
use anyhow::Error;

use crate::verify::bankai::mmr::BankaiMmr;
use crate::verify::bankai::stwo::verify_stwo_proof;
use alloy_primitives::hex::ToHexExt;

pub struct BeaconVerifier;

impl BeaconVerifier {
    pub async fn verify_header_proof(proof: &BeaconHeaderProof) -> Result<BeaconHeader, Error> {
        let bankai_block = verify_stwo_proof(&proof.block_proof)?;

        // Check the bankai block mmr root matches the mmr proof root
        match proof.mmr_proof.hashing_function {
            HashingFunctionDto::Keccak => {
                assert_eq!(
                    proof.mmr_proof.root,
                    format!("0x{}", bankai_block.beacon.mmr_root_keccak.encode_hex())
                );
            }
            HashingFunctionDto::Poseidon => {
                assert_eq!(
                    proof.mmr_proof.root,
                    format!(
                        "0x{}",
                        bankai_block.beacon.mmr_root_poseidon.encode_hex()
                    )
                );
            }
        }

        // Verify the mmr proof
        let mmr_proof_valid = BankaiMmr::verify_mmr_proof(proof.mmr_proof.clone()).await?;
        assert!(mmr_proof_valid);

        // Check the header hash matches the mmr proof header hash
        let hash = proof.header.tree_hash_root();
        assert_eq!(
            format!("0x{}", hash.encode_hex()),
            proof.mmr_proof.header_hash.clone()
        );

        Ok(proof.header.clone())
    }
}
