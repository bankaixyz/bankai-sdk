extern crate alloc;
use alloc::vec::Vec;

use alloy_primitives::{keccak256, B256, hex::FromHex};
use bankai_types::{
    fetch::evm::MmrProof, proofs::{HashingFunctionDto, MmrProofDto}, utils::mmr::hash_to_leaf
};
use starknet_crypto::{Felt, poseidon_hash};

use crate::VerifyError;

pub struct CairoLikeMmr;

impl CairoLikeMmr {
    pub fn verify_mmr_proof(proof: &MmrProof) -> Result<bool, VerifyError> {
        assert_mmr_size_is_valid(proof.elements_count as usize)?;

        let expected_peaks_len = compute_expected_peaks_len(proof.elements_count as usize)?;
        if proof.peaks.len() != expected_peaks_len {
            return Err(VerifyError::InvalidMmrTree);
        }

        // Parse leaf according to hashing function
        match proof.hashing_function {
            HashingFunctionDto::Keccak => verify_keccak(proof),
            HashingFunctionDto::Poseidon => verify_poseidon(proof),
        }
        // Ok(true)
    }
}

fn verify_keccak(proof: &MmrProof) -> Result<bool, VerifyError> {
    let elements_count = proof.elements_count as usize;
    let element_index = proof.elements_index as usize;

    let (peak_index, peak_height) = get_peak_info(elements_count, element_index)
        .ok_or(VerifyError::InvalidMmrProof)?;

    if element_index != elements_count {
        if proof.path.len() != peak_height { return Err(VerifyError::InvalidMmrProof); }
    } else if !proof.path.is_empty() {
        return Err(VerifyError::InvalidMmrProof);
    }
    let leaf = hash_to_leaf(proof.header_hash.clone(), &HashingFunctionDto::Keccak);

    let computed_peak = if element_index == elements_count {
        leaf
    } else {
        hash_subtree_path_keccak(leaf, 0, element_index, &proof.path)
    };
    println!("computed_peak: {}", computed_peak);

    
    println!("peaks: {:?}", proof.peaks);
    println!("peaks[peak_index]: {}", proof.peaks[peak_index]);
    println!("computed_peak: {}", computed_peak);
    if proof.peaks[peak_index] != computed_peak { return Err(VerifyError::InvalidMmrProof); }

    let bag = bag_peaks_left_to_right_keccak(&proof.peaks);
    let root = hash_mmr_root_keccak(elements_count as u128, &bag);
    if root != proof.root { return Err(VerifyError::InvalidMmrRoot); }
    Ok(true)
}

fn hash_subtree_path_keccak(element: B256, height: usize, position: usize, path: &[B256]) -> B256 {
    if path.is_empty() { return element; }
    let position_height = compute_height_pre_alloc_pow2(position);
    let next_height = compute_height_pre_alloc_pow2(position + 1);
    if next_height == position_height + 1 {
        let element = hash_pair(&path[0], &element);
        hash_subtree_path_keccak(element, height + 1, position + 1, &path[1..])
    } else {
        let element = hash_pair(&element, &path[0]);
        let position = position + (1usize << (height + 1));
        hash_subtree_path_keccak(element, height + 1, position, &path[1..])
    }
}

fn hash_subtree_path_poseidon(element: Felt, height: usize, position: usize, path: &[Felt]) -> Felt {
    if path.is_empty() { return element; }
    let position_height = compute_height_pre_alloc_pow2(position);
    let next_height = compute_height_pre_alloc_pow2(position + 1);
    if next_height == position_height + 1 {
        let element = poseidon_hash(path[0], element);
        hash_subtree_path_poseidon(element, height + 1, position + 1, &path[1..])
    } else {
        let element = poseidon_hash(element, path[0]);
        let position = position + (1usize << (height + 1));
        hash_subtree_path_poseidon(element, height + 1, position, &path[1..])
    }
}


fn verify_poseidon(proof: &MmrProof) -> Result<bool, VerifyError> {
    let elements_count = proof.elements_count as usize;
    let element_index = proof.elements_index as usize;

    let (peak_index, peak_height) = get_peak_info(elements_count, element_index)
        .ok_or(VerifyError::InvalidMmrProof)?;

    if element_index != elements_count {
        if proof.path.len() != peak_height { return Err(VerifyError::InvalidMmrProof); }
    } else if !proof.path.is_empty() {
        return Err(VerifyError::InvalidMmrProof);
    }

    println!("peak_index: {}", peak_index);
    // Leaf as Felt using the same method as hash_to_leaf used for Poseidon in types

    
    let leaf_bytes = hash_to_leaf(proof.header_hash.clone(), &HashingFunctionDto::Poseidon);
    let leaf = felt_from_b256(&leaf_bytes);

    let siblings: Vec<Felt> = proof
        .path
        .iter()
        .map(|h| felt_from_b256(&h))
        .collect::<Vec<Felt>>();

    let computed_peak = if element_index == elements_count {
        leaf
    } else {
        hash_subtree_path_poseidon(leaf, 0, element_index, &siblings)
    };

    println!("computed_peak: {}", computed_peak);

    let peaks: Vec<Felt> = proof
        .peaks
        .iter()
        .map(|h| felt_from_b256(&h))
        .collect::<Vec<Felt>>();
    println!("peaks: {:?}", peaks);
    println!("peaks[peak_index]: {}", peaks[peak_index]);
    println!("computed_peak: {}", computed_peak);
    if peaks[peak_index] != computed_peak { return Err(VerifyError::InvalidMmrProof); }

    let bag = bag_peaks_left_to_right_poseidon(&peaks);
    let root = mmr_root_poseidon(elements_count as u128, &bag);

    // compare as Felt parsed from hex
    if root != felt_from_b256(&proof.root) { return Err(VerifyError::InvalidMmrRoot); }
    Ok(true)
}

fn parse_b256(hex: &str) -> Option<B256> { B256::from_hex(hex).ok() }

fn felt_from_b256(b: &B256) -> Felt {
    Felt::from_bytes_be_slice(b.as_slice())
}

fn hash_pair(left: &B256, right: &B256) -> B256 {
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(left.as_slice());
    buf[32..].copy_from_slice(right.as_slice());
    keccak256(buf)
}

fn bag_peaks_left_to_right_keccak(peaks: &[B256]) -> B256 {
    match peaks.len() {
        0 => B256::ZERO,
        1 => peaks[0],
        _ => {
            // H(peak1, H(peak2, ... H(peakN)))
            let mut acc = *peaks.last().unwrap();
            for i in (0..peaks.len() - 1).rev() {
                acc = hash_pair(&peaks[i], &acc);
            }
            acc
        }
    }
}

fn bag_peaks_left_to_right_poseidon(peaks: &[Felt]) -> Felt {
    match peaks.len() {
        0 => Felt::ZERO,
        1 => peaks[0],
        _ => {
            let mut acc = *peaks.last().unwrap();
            for i in (0..peaks.len() - 1).rev() {
                acc = poseidon_hash(peaks[i], acc);
            }
            acc
        }
    }
}

fn hash_mmr_root_keccak(elements_count: u128, bag: &B256) -> B256 {
    let mut size_be = [0u8; 32];
    size_be[16..].copy_from_slice(&elements_count.to_be_bytes());
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&size_be);
    buf[32..].copy_from_slice(bag.as_slice());
    keccak256(buf)
}

fn mmr_root_poseidon(elements_count: u128, bag: &Felt) -> Felt {
    // size as Felt: use big-endian bytes of u128
    let size_felt = Felt::from_bytes_be_slice(&elements_count.to_be_bytes());
    poseidon_hash(size_felt, *bag)
}

fn assert_mmr_size_is_valid(x: usize) -> Result<(), VerifyError> {
    println!("assert_mmr_size_is_valid: {}", x);
    if x == 0 { return Err(VerifyError::InvalidMmrTree); }
    // inner: decompose x into distinct peaks of form (2^k - 1)
    let mut n = x;
    let mut prev_peak = 0usize;
    while n > 0 {
        let i = bit_length(n);
        if i == 0 { return Err(VerifyError::InvalidMmrTree); }
        let peak_tmp = (1usize << i) - 1;
        let peak = if n + 1 <= peak_tmp { (1usize << (i - 1)) - 1 } else { peak_tmp };
        if peak == 0 || peak == prev_peak { return Err(VerifyError::InvalidMmrTree); }
        n -= peak;
        prev_peak = peak;
    }
    println!("assert_mmr_size_is_valid: Ok");
    Ok(())
}

fn compute_expected_peaks_len(mmr_size: usize) -> Result<usize, VerifyError> {
    assert_mmr_size_is_valid(mmr_size)?;
    let mut n = mmr_size;
    let mut count = 0usize;
    let mut prev_peak = 0usize;
    while n > 0 {
        let i = bit_length(n);
        let peak_tmp = (1usize << i) - 1;
        let peak = if n + 1 <= peak_tmp { (1usize << (i - 1)) - 1 } else { peak_tmp };
        if peak == 0 || peak == prev_peak { return Err(VerifyError::InvalidMmrTree); }
        count += 1;
        n -= peak;
        prev_peak = peak;
    }
    Ok(count)
}

fn get_peak_info(mut elements_count: usize, mut element_index: usize) -> Option<(usize, usize)> {
    if element_index == 0 || element_index > elements_count { return None; }
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

fn compute_height_pre_alloc_pow2(mut x: usize) -> usize {
    // Mirrors Cairo compute_height_pre_alloc_pow2 exactly
    // Assumes x >= 1
    loop {
        let bit_length = bit_length(x);
        if bit_length == 0 { return 0; }
        let n = 1usize << (bit_length - 1);
        let n2 = 1usize << bit_length; // N
        if x == n2 - 1 {
            return bit_length - 1;
        } else {
            // Jump left: x = x - n + 1
            x = x - n + 1;
        }
    }
}

fn bit_length(n: usize) -> usize { (usize::BITS as usize) - n.leading_zeros() as usize }


