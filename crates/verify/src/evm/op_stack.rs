use alloy_primitives::FixedBytes;
use bankai_core::merkle::KeccakHasher;
use bankai_types::common::HashingFunction;
use bankai_types::inputs::evm::op_stack::{OpStackHeaderProof, OpStackMerkleProof};
use bankai_types::results::evm::execution::ExecutionHeader;

use crate::bankai::mmr::MmrVerifier;
use crate::VerifyError;

pub struct OpStackVerifier;

impl OpStackVerifier {
    pub fn verify_merkle_proof(
        proof: &OpStackMerkleProof,
        op_chains_root: FixedBytes<32>,
    ) -> Result<(), VerifyError> {
        let computed_root = bankai_core::merkle::hash_path::<KeccakHasher>(
            &proof.path,
            proof.leaf_hash,
            proof.merkle_leaf_index,
        );
        if computed_root != op_chains_root {
            return Err(VerifyError::InvalidMerkleProof);
        }
        Ok(())
    }

    pub fn verify_header_proof(
        proof: &OpStackHeaderProof,
        op_chains_root: FixedBytes<32>,
        hashing_function: HashingFunction,
    ) -> Result<ExecutionHeader, VerifyError> {
        let computed_leaf = proof.snapshot.commitment_leaf_hash();
        if computed_leaf != proof.merkle_proof.leaf_hash {
            return Err(VerifyError::InvalidMerkleProof);
        }
        
        // verify the snapshopt via merkle proof
        Self::verify_merkle_proof(&proof.merkle_proof, op_chains_root)?;

        // select the correct mmr root based on the hashing function
        let mmr_root = match hashing_function {
            HashingFunction::Keccak => proof.snapshot.mmr_root_keccak,
            HashingFunction::Poseidon => proof.snapshot.mmr_root_poseidon,
        };

        // ensure the mmr proof, uses the correct root
        if proof.mmr_proof.root != mmr_root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        // verify the mmr proof
        MmrVerifier::verify_mmr_proof(&proof.mmr_proof)?;

        // verify the header hash
        let header_hash = proof.header.hash_slow();
        if header_hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone().into())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Sealable;
    use bankai_core::mmr;
    use bankai_types::block::OpChainClient;
    use bankai_types::common::HashingFunction;
    use bankai_types::inputs::evm::op_stack::{OpStackHeaderProof, OpStackMerkleProof};
    use bankai_types::utils::mmr::hash_to_leaf;

    use super::*;

    fn snapshot() -> OpChainClient {
        OpChainClient {
            chain_id: 10,
            block_number: 42,
            header_hash: FixedBytes::from([7u8; 32]),
            l1_submission_block: 99,
            mmr_root_keccak: FixedBytes::ZERO,
            mmr_root_poseidon: FixedBytes::ZERO,
        }
    }

    fn single_leaf_mmr_proof(
        header_hash: FixedBytes<32>,
        hashing_function: HashingFunction,
    ) -> MmrProof {
        let leaf = hash_to_leaf(header_hash, &hashing_function).0;
        let root = match hashing_function {
            HashingFunction::Keccak => {
                mmr::calculate_root_hash(&mmr::KeccakHasher::new(), 1, &[leaf]).unwrap()
            }
            HashingFunction::Poseidon => {
                mmr::calculate_root_hash(&mmr::PoseidonHasher::new(), 1, &[leaf]).unwrap()
            }
        };

        MmrProof {
            network_id: 10,
            block_number: 42,
            hashing_function,
            header_hash,
            root: FixedBytes::from(root),
            elements_index: 1,
            elements_count: 1,
            path: vec![],
            peaks: vec![FixedBytes::from(leaf)],
        }
    }

    #[test]
    fn verifies_snapshot_membership_and_mmr_proof() {
        let mut snapshot = snapshot();
        let mut consensus_header = alloy_consensus::Header::default();
        consensus_header.number = 42;
        let header: alloy_rpc_types_eth::Header<alloy_consensus::Header> =
            alloy_rpc_types_eth::Header::from_consensus(consensus_header.seal_slow(), None, None);
        let header_hash = header.hash_slow();
        let mmr_proof = single_leaf_mmr_proof(header_hash, HashingFunction::Keccak);
        snapshot.mmr_root_keccak = mmr_proof.root;
        snapshot.header_hash = header_hash;
        let leaf_hash = snapshot.commitment_leaf_hash();
        let proof = OpStackHeaderProof {
            header,
            snapshot: snapshot.clone(),
            merkle_proof: OpStackMerkleProof {
                chain_id: snapshot.chain_id,
                merkle_leaf_index: 0,
                leaf_hash,
                root: leaf_hash,
                path: vec![],
            },
            mmr_proof,
        };

        let verified =
            OpStackVerifier::verify_header_proof(&proof, leaf_hash, HashingFunction::Keccak)
                .unwrap();

        assert_eq!(verified.number, snapshot.block_number);
    }

    #[test]
    fn rejects_wrong_merkle_root() {
        let snapshot = snapshot();
        let proof = OpStackMerkleProof {
            chain_id: snapshot.chain_id,
            merkle_leaf_index: 0,
            leaf_hash: snapshot.commitment_leaf_hash(),
            root: FixedBytes::from([1u8; 32]),
            path: vec![],
        };

        assert_eq!(
            OpStackVerifier::verify_merkle_proof(&proof, FixedBytes::from([2u8; 32])),
            Err(VerifyError::InvalidMerkleProof)
        );
    }
}
