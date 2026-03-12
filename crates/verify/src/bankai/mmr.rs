use mmr;
use bankai_types::{common::HashingFunction, inputs::evm::MmrProof, utils::mmr::hash_to_leaf};

use crate::VerifyError;

pub struct MmrVerifier;

impl MmrVerifier {
    pub fn verify_mmr_proof(proof: &MmrProof) -> Result<(), VerifyError> {
        let leaf = hash_to_leaf(proof.header_hash, &proof.hashing_function).0;
        let mmr_proof = mmr::Proof {
            element_index: proof.elements_index,
            element_hash: leaf,
            siblings_hashes: proof.path.iter().map(|hash| hash.0).collect(),
            peaks_hashes: proof.peaks.iter().map(|hash| hash.0).collect(),
            elements_count: proof.elements_count,
        };

        // Ensure the merkle path recreates a specific peak hash
        let valid = with_hasher(
            proof.hashing_function,
            |hasher| mmr::verify_proof_stateless(hasher, &mmr_proof, leaf),
            |hasher| mmr::verify_proof_stateless(hasher, &mmr_proof, leaf),
        )
        .map_err(map_mmr_error)?;

        if !valid {
            return Err(VerifyError::InvalidMmrProof);
        }

        // ensure the peaks create the expected root
        let computed_root = with_hasher(
            proof.hashing_function,
            |hasher| {
                mmr::calculate_root_hash(hasher, proof.elements_count, &mmr_proof.peaks_hashes)
            },
            |hasher| {
                mmr::calculate_root_hash(hasher, proof.elements_count, &mmr_proof.peaks_hashes)
            },
        )
        .map_err(map_mmr_error)?;

        if computed_root != proof.root.0 {
            return Err(VerifyError::InvalidMmrRoot);
        }

        Ok(())
    }
}

fn with_hasher<T, FKeccak, FPoseidon>(
    hashing_function: HashingFunction,
    keccak: FKeccak,
    poseidon: FPoseidon,
) -> T
where
    FKeccak: FnOnce(&mmr::KeccakHasher) -> T,
    FPoseidon: FnOnce(&mmr::PoseidonHasher) -> T,
{
    match hashing_function {
        HashingFunction::Keccak => keccak(&mmr::KeccakHasher::new()),
        HashingFunction::Poseidon => poseidon(&mmr::PoseidonHasher::new()),
    }
}

fn map_mmr_error(error: mmr::MmrError) -> VerifyError {
    match error {
        mmr::MmrError::InvalidElementCount
        | mmr::MmrError::InvalidPeaksCount
        | mmr::MmrError::InvalidPeaksCountForElements
        | mmr::MmrError::Overflow => VerifyError::InvalidMmrTree,
        mmr::MmrError::InvalidElementIndex | mmr::MmrError::Hasher(_) => {
            VerifyError::InvalidMmrProof
        }
        _ => VerifyError::InvalidMmrProof,
    }
}

#[cfg(test)]
mod tests {
    use ::mmr as external_mmr;
    use alloy_primitives::FixedBytes;
    use bankai_types::utils::mmr::hash_to_leaf;

    use super::*;

    fn base_proof(hashing_function: HashingFunction) -> MmrProof {
        let header_hash = FixedBytes::from([7u8; 32]);
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
            network_id: 1,
            block_number: 1,
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
    fn verifies_valid_keccak_proof() {
        let proof = base_proof(HashingFunction::Keccak);
        assert_eq!(MmrVerifier::verify_mmr_proof(&proof), Ok(()));
    }

    #[test]
    fn verifies_valid_poseidon_proof() {
        let proof = base_proof(HashingFunction::Poseidon);
        assert_eq!(MmrVerifier::verify_mmr_proof(&proof), Ok(()));
    }

    #[test]
    fn wrong_root_maps_to_invalid_mmr_root() {
        let mut proof = base_proof(HashingFunction::Keccak);
        proof.root = FixedBytes::from([9u8; 32]);
        assert_eq!(
            MmrVerifier::verify_mmr_proof(&proof),
            Err(VerifyError::InvalidMmrRoot)
        );
    }

    #[test]
    fn invalid_peaks_count_maps_to_invalid_mmr_tree() {
        let mut proof = base_proof(HashingFunction::Keccak);
        proof.peaks.clear();
        assert_eq!(
            MmrVerifier::verify_mmr_proof(&proof),
            Err(VerifyError::InvalidMmrTree)
        );
    }

    #[test]
    fn invalid_element_count_maps_to_invalid_mmr_tree() {
        let mut proof = base_proof(HashingFunction::Keccak);
        proof.elements_count = 2;
        assert_eq!(
            MmrVerifier::verify_mmr_proof(&proof),
            Err(VerifyError::InvalidMmrTree)
        );
    }

    #[test]
    fn invalid_element_index_maps_to_invalid_mmr_proof() {
        let mut proof = base_proof(HashingFunction::Keccak);
        proof.elements_index = 0;
        assert_eq!(
            MmrVerifier::verify_mmr_proof(&proof),
            Err(VerifyError::InvalidMmrProof)
        );
    }

    #[tokio::test]
    async fn tampered_path_maps_to_invalid_mmr_proof() {
        let store = external_mmr::InMemoryStore::new();
        let hasher = std::sync::Arc::new(external_mmr::KeccakHasher::new());
        let mut tree = external_mmr::Mmr::new(store, hasher, None).unwrap();
        let header_hash = FixedBytes::from([3u8; 32]);
        let other_hash = FixedBytes::from([4u8; 32]);
        let third_hash = FixedBytes::from([5u8; 32]);

        tree.append(hash_to_leaf(header_hash, &HashingFunction::Keccak).0)
            .await
            .unwrap();
        tree.append(hash_to_leaf(other_hash, &HashingFunction::Keccak).0)
            .await
            .unwrap();
        tree.append(hash_to_leaf(third_hash, &HashingFunction::Keccak).0)
            .await
            .unwrap();

        let generated = tree.get_proof(1, None).await.unwrap();
        let root = tree.get_root_hash().await.unwrap().unwrap();
        let mut path = generated.siblings_hashes;
        path[0] = [0u8; 32];

        let proof = MmrProof {
            network_id: 1,
            block_number: 1,
            hashing_function: HashingFunction::Keccak,
            header_hash,
            root: FixedBytes::from(root),
            elements_index: generated.element_index,
            elements_count: generated.elements_count,
            path: path.into_iter().map(FixedBytes::from).collect(),
            peaks: generated
                .peaks_hashes
                .into_iter()
                .map(FixedBytes::from)
                .collect(),
        };

        assert_eq!(
            MmrVerifier::verify_mmr_proof(&proof),
            Err(VerifyError::InvalidMmrProof)
        );
    }
}
