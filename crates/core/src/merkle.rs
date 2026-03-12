use alloc::{vec, vec::Vec};

use crate::error::CoreError;
use alloy_primitives::FixedBytes;
use sha2::{Digest, Sha256};
use tiny_keccak::{Hasher, Keccak};

#[cfg(feature = "poseidon")]
use crate::utils::{to_felt, to_felt252};
#[cfg(feature = "poseidon")]
use cairo_vm_base::vm::cairo_vm::Felt252;
#[cfg(feature = "poseidon")]
use starknet_crypto::poseidon_hash;

pub trait MerkleHasher {
    type Hash: Clone + PartialEq + Eq;

    fn hash_pair(left: &Self::Hash, right: &Self::Hash) -> Self::Hash;
    fn zero() -> Self::Hash;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Sha256Hasher;

#[derive(Debug, Clone, Copy, Default)]
pub struct KeccakHasher;

#[cfg(feature = "poseidon")]
#[derive(Debug, Clone, Copy, Default)]
pub struct PoseidonHasher;

impl MerkleHasher for Sha256Hasher {
    type Hash = FixedBytes<32>;

    fn hash_pair(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
        let mut data = [0u8; 64];
        data[..32].copy_from_slice(left.as_slice());
        data[32..].copy_from_slice(right.as_slice());
        FixedBytes::from(<[u8; 32]>::from(Sha256::digest(data)))
    }

    fn zero() -> Self::Hash {
        FixedBytes::ZERO
    }
}

impl MerkleHasher for KeccakHasher {
    type Hash = FixedBytes<32>;

    fn hash_pair(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
        let mut data = [0u8; 64];
        data[..32].copy_from_slice(left.as_slice());
        data[32..].copy_from_slice(right.as_slice());
        FixedBytes::from(keccak256(&data))
    }

    fn zero() -> Self::Hash {
        FixedBytes::ZERO
    }
}

#[cfg(feature = "poseidon")]
impl MerkleHasher for PoseidonHasher {
    type Hash = Felt252;

    fn hash_pair(left: &Self::Hash, right: &Self::Hash) -> Self::Hash {
        to_felt252(poseidon_hash(to_felt(*left), to_felt(*right)))
    }

    fn zero() -> Self::Hash {
        to_felt252(starknet_crypto::Felt::ZERO)
    }
}

pub fn compute_root<H>(leaves: &[H::Hash]) -> H::Hash
where
    H: MerkleHasher,
{
    let tree = build_tree::<H>(leaves);
    tree.last().unwrap()[0].clone()
}

pub fn hash_path<H>(path: &[H::Hash], leaf: H::Hash, index: u64) -> H::Hash
where
    H: MerkleHasher,
{
    let mut value = leaf;
    let mut current_index = index;

    for sibling in path {
        value = if current_index % 2 == 0 {
            H::hash_pair(&value, sibling)
        } else {
            H::hash_pair(sibling, &value)
        };
        current_index /= 2;
    }

    value
}

pub fn generate_path<H>(leaves: &[H::Hash], leaf_index: usize) -> Result<Vec<H::Hash>, CoreError>
where
    H: MerkleHasher,
{
    if leaf_index >= leaves.len() {
        return Err(CoreError::InvalidMerkleTree);
    }

    let tree = build_tree::<H>(leaves);
    let mut path = Vec::with_capacity(tree.len().saturating_sub(1));
    let mut current_index = leaf_index;

    for level in &tree[..tree.len().saturating_sub(1)] {
        let sibling_index = if current_index % 2 == 0 {
            current_index + 1
        } else {
            current_index - 1
        };
        path.push(level[sibling_index].clone());
        current_index /= 2;
    }

    Ok(path)
}

pub fn compute_paths<H>(leaves: &[H::Hash]) -> (H::Hash, Vec<Vec<H::Hash>>)
where
    H: MerkleHasher,
{
    let tree = build_tree::<H>(leaves);
    let root = tree.last().unwrap()[0].clone();
    let mut paths = Vec::with_capacity(leaves.len());

    for leaf_index in 0..leaves.len() {
        let mut path = Vec::with_capacity(tree.len().saturating_sub(1));
        let mut current_index = leaf_index;

        for level in &tree[..tree.len().saturating_sub(1)] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            path.push(level[sibling_index].clone());
            current_index /= 2;
        }

        paths.push(path);
    }

    (root, paths)
}

pub fn update_leaf<H>(
    path: &[H::Hash],
    old_leaf: H::Hash,
    new_leaf: H::Hash,
    index: u64,
    expected_root: H::Hash,
) -> Result<H::Hash, CoreError>
where
    H: MerkleHasher,
{
    let old_root = hash_path::<H>(path, old_leaf, index);
    if old_root != expected_root {
        return Err(CoreError::InvalidMerkleProof);
    }

    Ok(hash_path::<H>(path, new_leaf, index))
}

pub fn compute_updated_nodes<H>(
    leaf_index: u64,
    leaf_hash: H::Hash,
    path: &[H::Hash],
) -> Vec<((u32, u64), H::Hash)>
where
    H: MerkleHasher,
{
    let mut current_hash = leaf_hash;
    let mut current_index = leaf_index;
    let mut nodes = Vec::with_capacity(path.len() + 1);

    nodes.push(((0, current_index), current_hash.clone()));

    for (level, sibling) in path.iter().enumerate() {
        current_hash = if current_index % 2 == 0 {
            H::hash_pair(&current_hash, sibling)
        } else {
            H::hash_pair(sibling, &current_hash)
        };
        current_index /= 2;
        nodes.push((((level as u32) + 1, current_index), current_hash.clone()));
    }

    nodes
}

pub mod op_stack {
    use alloc::vec::Vec;

    use alloy_primitives::FixedBytes;

    use crate::error::CoreError;

    use super::{
        compute_root as generic_compute_root,
        compute_updated_nodes as generic_compute_updated_nodes, generate_path, hash_path,
        keccak256, update_leaf as generic_update_leaf, KeccakHasher,
    };

    pub const TREE_DEPTH: usize = 5;
    pub const MAX_CLIENTS: usize = 1 << TREE_DEPTH;

    fn u64_to_word(value: u64) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[24..32].copy_from_slice(&value.to_be_bytes());
        out
    }

    fn bytes32_to_word(value: &FixedBytes<32>) -> [u8; 32] {
        let mut out = [0u8; 32];
        out.copy_from_slice(value.as_slice());
        out
    }

    pub fn leaf_hash(
        chain_id: u64,
        block_number: u64,
        header_hash: FixedBytes<32>,
        l1_submission_block: u64,
        mmr_root_keccak: FixedBytes<32>,
        mmr_root_poseidon: FixedBytes<32>,
    ) -> FixedBytes<32> {
        let words = [
            u64_to_word(chain_id),
            u64_to_word(block_number),
            bytes32_to_word(&header_hash),
            u64_to_word(l1_submission_block),
            bytes32_to_word(&mmr_root_keccak),
            bytes32_to_word(&mmr_root_poseidon),
        ];

        let mut preimage = Vec::with_capacity(words.len() * 32);
        for word in words {
            preimage.extend_from_slice(&word);
        }

        FixedBytes::from_slice(keccak256(&preimage).as_slice())
    }

    pub fn empty_leaf_hash() -> FixedBytes<32> {
        leaf_hash(
            0,
            0,
            FixedBytes::ZERO,
            0,
            FixedBytes::ZERO,
            FixedBytes::ZERO,
        )
    }

    fn padded_leaves(leaves: &[FixedBytes<32>]) -> Result<Vec<FixedBytes<32>>, CoreError> {
        if leaves.len() > MAX_CLIENTS {
            return Err(CoreError::InvalidOpStackCommitment);
        }

        let mut padded = leaves.to_vec();
        padded.resize(MAX_CLIENTS, empty_leaf_hash());
        Ok(padded)
    }

    pub fn empty_root() -> FixedBytes<32> {
        compute_root(&[]).expect("fixed-size empty OP stack tree is valid")
    }

    pub fn compute_root(leaves: &[FixedBytes<32>]) -> Result<FixedBytes<32>, CoreError> {
        Ok(generic_compute_root::<KeccakHasher>(&padded_leaves(
            leaves,
        )?))
    }

    pub fn generate_proof(
        leaves: &[FixedBytes<32>],
        target_leaf_index: u64,
    ) -> Result<(FixedBytes<32>, FixedBytes<32>, Vec<FixedBytes<32>>), CoreError> {
        let leaves = padded_leaves(leaves)?;
        let leaf_index =
            usize::try_from(target_leaf_index).map_err(|_| CoreError::InvalidOpStackCommitment)?;
        if leaf_index >= MAX_CLIENTS {
            return Err(CoreError::InvalidOpStackCommitment);
        }

        let path = generate_path::<KeccakHasher>(&leaves, leaf_index)?;
        let leaf_hash = leaves[leaf_index];
        let root = generic_compute_root::<KeccakHasher>(&leaves);
        Ok((leaf_hash, root, path))
    }

    pub fn verify_proof(
        path: &[FixedBytes<32>],
        leaf_hash: FixedBytes<32>,
        leaf_index: u64,
        expected_root: FixedBytes<32>,
    ) -> Result<(), CoreError> {
        let computed_root = hash_path::<KeccakHasher>(path, leaf_hash, leaf_index);
        if computed_root != expected_root {
            return Err(CoreError::InvalidMerkleProof);
        }
        Ok(())
    }

    pub fn update_leaf(
        path: &[FixedBytes<32>],
        old_leaf_hash: FixedBytes<32>,
        new_leaf_hash: FixedBytes<32>,
        leaf_index: u64,
        expected_root: FixedBytes<32>,
    ) -> Result<FixedBytes<32>, CoreError> {
        generic_update_leaf::<KeccakHasher>(
            path,
            old_leaf_hash,
            new_leaf_hash,
            leaf_index,
            expected_root,
        )
    }

    pub fn compute_updated_nodes(
        leaf_index: u64,
        leaf_hash: FixedBytes<32>,
        path: &[FixedBytes<32>],
    ) -> Vec<((u32, u64), FixedBytes<32>)> {
        generic_compute_updated_nodes::<KeccakHasher>(leaf_index, leaf_hash, path)
    }
}

fn build_tree<H>(leaves: &[H::Hash]) -> Vec<Vec<H::Hash>>
where
    H: MerkleHasher,
{
    let tree_size = leaves.len().max(1).next_power_of_two();
    let mut current_level = leaves.to_vec();

    while current_level.len() < tree_size {
        current_level.push(H::zero());
    }

    if current_level.is_empty() {
        current_level.push(H::zero());
    }

    let mut tree = vec![current_level.clone()];

    while current_level.len() > 1 {
        let mut next_level = Vec::with_capacity(current_level.len() / 2);
        for pair in current_level.chunks(2) {
            next_level.push(H::hash_pair(&pair[0], &pair[1]));
        }
        tree.push(next_level.clone());
        current_level = next_level;
    }

    tree
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    let mut keccak = Keccak::v256();
    keccak.update(bytes);
    keccak.finalize(&mut hash);
    hash
}

#[cfg(test)]
mod tests {
    use super::{
        compute_paths, compute_root, compute_updated_nodes, generate_path, hash_path, update_leaf,
        KeccakHasher, MerkleHasher,
    };
    use crate::error::CoreError;
    use alloy_primitives::FixedBytes;

    fn leaf(byte: u8) -> FixedBytes<32> {
        FixedBytes::from([byte; 32])
    }

    fn compute_keccak_root(leaves: &[FixedBytes<32>]) -> FixedBytes<32> {
        compute_root::<KeccakHasher>(leaves)
    }

    #[test]
    fn keccak_generate_path_and_hash_path_round_trip() {
        let leaves = vec![leaf(1), leaf(2), leaf(3)];
        let expected_root = compute_keccak_root(&leaves);

        for (idx, item) in leaves.iter().cloned().enumerate() {
            let path = generate_path::<KeccakHasher>(&leaves, idx).unwrap();
            let computed_root = hash_path::<KeccakHasher>(&path, item, idx as u64);
            assert_eq!(computed_root, expected_root);
        }
    }

    #[test]
    fn keccak_compute_paths_round_trip() {
        let leaves = vec![leaf(1), leaf(2), leaf(3), leaf(4), leaf(5)];
        let (root, paths) = compute_paths::<KeccakHasher>(&leaves);

        assert_eq!(root, compute_keccak_root(&leaves));
        assert_eq!(paths.len(), leaves.len());

        for (idx, (leaf, path)) in leaves.iter().cloned().zip(paths.iter()).enumerate() {
            assert_eq!(hash_path::<KeccakHasher>(path, leaf, idx as u64), root);
        }
    }

    #[test]
    fn keccak_generate_path_invalid_index() {
        let result = generate_path::<KeccakHasher>(&[leaf(1), leaf(2)], 2);
        assert!(matches!(result, Err(CoreError::InvalidMerkleTree)));
    }

    #[test]
    fn keccak_update_leaf_returns_new_root() {
        let mut leaves = vec![leaf(1), leaf(2), leaf(3), leaf(4)];
        let index = 2usize;
        let old_root = compute_keccak_root(&leaves);
        let path = generate_path::<KeccakHasher>(&leaves, index).unwrap();

        let new_leaf = leaf(9);
        let updated_root =
            update_leaf::<KeccakHasher>(&path, leaves[index], new_leaf, index as u64, old_root)
                .unwrap();

        leaves[index] = new_leaf;
        let expected_updated_root = compute_keccak_root(&leaves);
        assert_eq!(updated_root, expected_updated_root);
    }

    #[test]
    fn keccak_update_leaf_rejects_invalid_proof() {
        let leaves = vec![leaf(1), leaf(2), leaf(3)];
        let path = generate_path::<KeccakHasher>(&leaves, 0).unwrap();
        let err = update_leaf::<KeccakHasher>(&path, leaves[0], leaf(9), 0, leaf(255)).unwrap_err();
        assert!(matches!(err, CoreError::InvalidMerkleProof));
    }

    #[test]
    fn keccak_compute_updated_nodes_tracks_leaf_and_ancestors() {
        let leaves = vec![leaf(1), leaf(2), leaf(3), leaf(4)];
        let index = 2usize;
        let path = generate_path::<KeccakHasher>(&leaves, index).unwrap();
        let new_leaf = leaf(9);

        let nodes = compute_updated_nodes::<KeccakHasher>(index as u64, new_leaf, &path);
        assert_eq!(nodes.len(), path.len() + 1);
        assert_eq!(nodes[0], ((0, index as u64), new_leaf));

        let mut expected_hash = new_leaf;
        let mut expected_index = index as u64;
        for (level, sibling) in path.iter().enumerate() {
            expected_hash = if expected_index % 2 == 0 {
                KeccakHasher::hash_pair(&expected_hash, sibling)
            } else {
                KeccakHasher::hash_pair(sibling, &expected_hash)
            };
            expected_index /= 2;
            assert_eq!(
                nodes[level + 1],
                (((level as u32) + 1, expected_index), expected_hash)
            );
        }
    }

    #[test]
    fn keccak_compute_updated_nodes_last_entry_matches_new_root() {
        let leaves = vec![leaf(1), leaf(2), leaf(3), leaf(4), leaf(5)];
        let index = 4usize;
        let path = generate_path::<KeccakHasher>(&leaves, index).unwrap();
        let new_leaf = leaf(8);

        let nodes = compute_updated_nodes::<KeccakHasher>(index as u64, new_leaf, &path);
        let root_from_path = hash_path::<KeccakHasher>(&path, new_leaf, index as u64);
        let root_from_nodes = nodes.last().unwrap().1;

        assert_eq!(root_from_nodes, root_from_path);
    }

    mod op_stack {
        use alloy_primitives::{hex::FromHex, FixedBytes};

        use crate::{
            error::CoreError,
            merkle::op_stack::{
                compute_root, empty_leaf_hash, empty_root, generate_proof, leaf_hash, verify_proof,
                MAX_CLIENTS,
            },
        };

        fn sample_leaf(seed: u8) -> FixedBytes<32> {
            leaf_hash(
                seed as u64,
                (seed as u64) + 10,
                FixedBytes::from([seed; 32]),
                (seed as u64) + 100,
                FixedBytes::from([seed.wrapping_add(1); 32]),
                FixedBytes::from([seed.wrapping_add(2); 32]),
            )
        }

        #[test]
        fn empty_leaf_hash_is_stable() {
            let expected = FixedBytes::<32>::from_hex(
                "0x1e990e27f0d7976bf2adbd60e20384da0125b76e2885a96aa707bcb054108b0d",
            )
            .unwrap();

            assert_eq!(empty_leaf_hash(), expected);
        }

        #[test]
        fn empty_root_matches_cairo_constant() {
            let expected = FixedBytes::<32>::from_hex(
                "0xd686d974150e54f427421b5805b6464c7736dcf70944067195505a19e433d326",
            )
            .unwrap();

            assert_eq!(empty_root(), expected);
        }

        #[test]
        fn root_pads_tail_with_empty_leaves() {
            let mut leaves = vec![sample_leaf(1), sample_leaf(2), sample_leaf(3)];
            let compact_root = compute_root(&leaves).unwrap();

            leaves.resize(MAX_CLIENTS, empty_leaf_hash());

            assert_eq!(compact_root, compute_root(&leaves).unwrap());
        }

        #[test]
        fn proof_round_trip_for_contiguous_leaves() {
            let leaves = [sample_leaf(1), sample_leaf(2), sample_leaf(3)];

            for target_index in [0, 1, 2] {
                let (leaf_hash, root, path) = generate_proof(&leaves, target_index).unwrap();
                verify_proof(&path, leaf_hash, target_index, root).unwrap();
            }
        }

        #[test]
        fn proof_round_trip_for_padded_empty_leaf() {
            let leaves = [sample_leaf(1), sample_leaf(2)];
            let (leaf_hash, root, path) = generate_proof(&leaves, 7).unwrap();

            assert_eq!(leaf_hash, empty_leaf_hash());
            verify_proof(&path, leaf_hash, 7, root).unwrap();
        }

        #[test]
        fn too_many_leaves_fail_fast() {
            let err = compute_root(&vec![empty_leaf_hash(); MAX_CLIENTS + 1]).unwrap_err();
            assert!(matches!(err, CoreError::InvalidOpStackCommitment));
        }

        #[test]
        fn commitment_leaf_hash_preserves_empty_leaf_semantics() {
            assert_eq!(
                leaf_hash(
                    0,
                    0,
                    FixedBytes::ZERO,
                    0,
                    FixedBytes::ZERO,
                    FixedBytes::ZERO
                ),
                empty_leaf_hash()
            );
        }

        #[test]
        fn verify_proof_rejects_wrong_root() {
            let leaves = [sample_leaf(1), sample_leaf(2), sample_leaf(3)];
            let (leaf_hash, _, path) = generate_proof(&leaves, 2).unwrap();
            let err = verify_proof(&path, leaf_hash, 2, FixedBytes::from([0xFF; 32])).unwrap_err();
            assert!(matches!(err, CoreError::InvalidMerkleProof));
        }
    }
}
