use crate::{
    error::CoreError,
    utils::{to_felt, to_felt252},
};
use alloy_primitives::FixedBytes;
use cairo_vm_base::vm::cairo_vm::Felt252;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use starknet_crypto::poseidon_hash;
use tiny_keccak::{Hasher, Keccak};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePath {
    pub leaf_index: u64,
    pub value: Felt252,
}

pub trait MerkleHasher {
    type Hash: Clone + PartialEq + Eq;

    fn hash_pair(left: &Self::Hash, right: &Self::Hash) -> Self::Hash;
    fn zero() -> Self::Hash;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Sha256Hasher;

#[derive(Debug, Clone, Copy, Default)]
pub struct KeccakHasher;

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
}
