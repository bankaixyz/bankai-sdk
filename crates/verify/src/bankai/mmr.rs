extern crate alloc;
use alloc::vec::Vec;

use alloy_primitives::{keccak256, B256};
use bankai_types::{fetch::evm::MmrProof, proofs::HashingFunctionDto, utils::mmr::hash_to_leaf};
use starknet_crypto::{poseidon_hash, Felt};

use crate::VerifyError;
trait MmrHasher {
    type Word: Copy + PartialEq;

    /// Convert a 32-byte value from proofs into the internal word type.
    fn from_b256(x: &B256) -> Self::Word;

    /// Hash an ordered pair `(left, right)`.
    fn hash_pair(left: &Self::Word, right: &Self::Word) -> Self::Word;

    /// Compute the leaf from the raw header hash.
    fn leaf_from_header_hash(header_hash: &B256) -> Self::Word;

    /// Bag peaks right-to-left: `H(peak1, H(peak2, ... H(peakN)))`.
    fn bag_peaks(peaks: &[Self::Word]) -> Self::Word {
        match peaks.len() {
            0 => Self::from_b256(&B256::ZERO),
            1 => peaks[0],
            _ => {
                let mut acc = *peaks.last().unwrap();
                for i in (0..peaks.len() - 1).rev() {
                    acc = Self::hash_pair(&peaks[i], &acc);
                }
                acc
            }
        }
    }

    /// Compute the MMR root from the elements count and bagged peaks.
    fn mmr_root(elements_count: u128, bag: &Self::Word) -> Self::Word;
}

/// Keccak-based hasher using `B256` as the word type.
struct KeccakHasher;

impl MmrHasher for KeccakHasher {
    type Word = B256;

    #[inline]
    fn from_b256(x: &B256) -> Self::Word {
        *x
    }

    #[inline]
    fn hash_pair(left: &Self::Word, right: &Self::Word) -> Self::Word {
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(left.as_slice());
        buf[32..].copy_from_slice(right.as_slice());
        keccak256(buf)
    }

    #[inline]
    fn leaf_from_header_hash(header_hash: &B256) -> Self::Word {
        hash_to_leaf(*header_hash, &HashingFunctionDto::Keccak)
    }

    #[inline]
    fn mmr_root(elements_count: u128, bag: &Self::Word) -> Self::Word {
        let mut size_be = [0u8; 32];
        size_be[16..].copy_from_slice(&elements_count.to_be_bytes());
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(&size_be);
        buf[32..].copy_from_slice(bag.as_slice());
        keccak256(buf)
    }
}

/// Poseidon-based hasher using `Felt` as the word type.
struct PoseidonHasher;

impl MmrHasher for PoseidonHasher {
    type Word = Felt;

    #[inline]
    fn from_b256(x: &B256) -> Self::Word {
        Felt::from_bytes_be_slice(x.as_slice())
    }

    #[inline]
    fn hash_pair(left: &Self::Word, right: &Self::Word) -> Self::Word {
        poseidon_hash(*left, *right)
    }

    #[inline]
    fn leaf_from_header_hash(header_hash: &B256) -> Self::Word {
        // Reuse existing leaf definition then convert to Felt
        let leaf_bytes = hash_to_leaf(*header_hash, &HashingFunctionDto::Poseidon);
        Felt::from_bytes_be_slice(leaf_bytes.as_slice())
    }

    #[inline]
    fn mmr_root(elements_count: u128, bag: &Self::Word) -> Self::Word {
        let size_felt = Felt::from_bytes_be_slice(&elements_count.to_be_bytes());
        poseidon_hash(size_felt, *bag)
    }
}

/// Iterative subtree path hashing generic over the hasher.
///
/// Replays the sibling path from the leaf to its peak, following the same
/// left/right rules as the Cairo implementation but without recursion.
fn hash_subtree_path_iter<H: MmrHasher>(
    mut element: H::Word,
    mut height: usize,
    mut position: usize,
    path: &[H::Word],
) -> H::Word {
    if path.is_empty() {
        return element;
    }
    for sibling in path.iter() {
        let position_height = compute_height(position);
        let next_height = compute_height(position + 1);
        if next_height == position_height + 1 {
            element = H::hash_pair(sibling, &element);
            position += 1;
        } else {
            element = H::hash_pair(&element, sibling);
            position += 1usize << (height + 1);
        }
        height += 1;
    }
    element
}

pub struct MmrVerifier;

impl MmrVerifier {
    /// Verifies a single MMR proof, dispatching to the appropriate hasher.
    pub fn verify_mmr_proof(proof: &MmrProof) -> Result<bool, VerifyError> {
        assert_mmr_size_is_valid(proof.elements_count as usize)?;

        let expected_peaks_len = compute_expected_peaks_len(proof.elements_count as usize)?;
        if proof.peaks.len() != expected_peaks_len {
            return Err(VerifyError::InvalidMmrTree);
        }

        match proof.hashing_function {
            HashingFunctionDto::Keccak => verify_with_hasher::<KeccakHasher>(proof),
            HashingFunctionDto::Poseidon => verify_with_hasher::<PoseidonHasher>(proof),
        }
    }
}

/// Generic verification with a concrete `MmrHasher` implementation.
fn verify_with_hasher<H: MmrHasher>(proof: &MmrProof) -> Result<bool, VerifyError> {
    let elements_count = proof.elements_count as usize;
    let element_index = proof.elements_index as usize;

    let (peak_index, peak_height) =
        get_peak_info(elements_count, element_index).ok_or(VerifyError::InvalidMmrProof)?;

    if element_index != elements_count {
        if proof.path.len() != peak_height {
            return Err(VerifyError::InvalidMmrProof);
        }
    } else if !proof.path.is_empty() {
        return Err(VerifyError::InvalidMmrProof);
    }

    let leaf = H::leaf_from_header_hash(&proof.header_hash);

    let computed_peak = if element_index == elements_count {
        leaf
    } else {
        let siblings: Vec<H::Word> = proof.path.iter().map(H::from_b256).collect();
        hash_subtree_path_iter::<H>(leaf, 0, element_index, &siblings)
    };

    let peaks: Vec<H::Word> = proof.peaks.iter().map(H::from_b256).collect();
    if peaks[peak_index] != computed_peak {
        return Err(VerifyError::InvalidMmrProof);
    }

    let bag = H::bag_peaks(&peaks);
    let root = H::mmr_root(elements_count as u128, &bag);
    let proof_root = H::from_b256(&proof.root);
    if root != proof_root {
        return Err(VerifyError::InvalidMmrRoot);
    }
    Ok(true)
}

/// Validates that an MMR size can be decomposed into distinct peaks of the form `(2^k - 1)`.
fn assert_mmr_size_is_valid(x: usize) -> Result<(), VerifyError> {
    if x == 0 {
        return Err(VerifyError::InvalidMmrTree);
    }

    let mut n = x;
    let mut prev_peak = 0usize;
    while n > 0 {
        let i = bit_length(n);
        if i == 0 {
            return Err(VerifyError::InvalidMmrTree);
        }
        let peak_tmp = (1usize << i) - 1;
        let peak = if n < peak_tmp {
            (1usize << (i - 1)) - 1
        } else {
            peak_tmp
        };
        if peak == 0 || peak == prev_peak {
            return Err(VerifyError::InvalidMmrTree);
        }
        n -= peak;
        prev_peak = peak;
    }
    Ok(())
}

/// Computes how many peaks an MMR of `mmr_size` elements should have.
fn compute_expected_peaks_len(mmr_size: usize) -> Result<usize, VerifyError> {
    assert_mmr_size_is_valid(mmr_size)?;
    let mut n = mmr_size;
    let mut count = 0usize;
    let mut prev_peak = 0usize;
    while n > 0 {
        let i = bit_length(n);
        let peak_tmp = (1usize << i) - 1;
        let peak = if n < peak_tmp {
            (1usize << (i - 1)) - 1
        } else {
            peak_tmp
        };
        if peak == 0 || peak == prev_peak {
            return Err(VerifyError::InvalidMmrTree);
        }
        count += 1;
        n -= peak;
        prev_peak = peak;
    }
    Ok(count)
}

/// Returns `(peak_index, peak_height)` for the 1-indexed `element_index` in an MMR with
/// `elements_count` total elements. The height is the number of edges from the leaf to the peak.
fn get_peak_info(mut elements_count: usize, mut element_index: usize) -> Option<(usize, usize)> {
    if element_index == 0 || element_index > elements_count {
        return None;
    }
    let mut mountain_height = bit_length(elements_count);
    let mut mountain_elements_count = (1usize << mountain_height) - 1;
    let mut mountain_index = 0usize;
    loop {
        if mountain_elements_count <= elements_count {
            if element_index <= mountain_elements_count {
                return Some((mountain_index, mountain_height.saturating_sub(1)));
            }
            elements_count -= mountain_elements_count;
            element_index -= mountain_elements_count;
            mountain_index += 1;
        }
        mountain_elements_count >>= 1;
        mountain_height = mountain_height.saturating_sub(1);
    }
}

/// Computes the height in the implicit binary tree for a 1-indexed position `x`.
///
/// This walks left in the implicit perfect binary tree until reaching a peak `(2^k - 1)` and returns `k - 1`.
fn compute_height(mut x: usize) -> usize {
    loop {
        let bit_len = bit_length(x);
        if bit_len == 0 {
            return 0;
        }
        let n = 1usize << (bit_len - 1);
        let n2 = 1usize << bit_len; // N
        if x == n2 - 1 {
            return bit_len - 1;
        } else {
            // Jump left: x = x - n + 1
            x = x - n + 1;
        }
    }
}

/// Returns the number of bits required to represent `n` (0 => 0, 1.. => floor(log2(n)) + 1).
fn bit_length(n: usize) -> usize {
    (usize::BITS as usize) - n.leading_zeros() as usize
}
