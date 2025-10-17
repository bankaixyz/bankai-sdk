extern crate alloc;
use alloc::vec::Vec;

use alloy_primitives::{keccak256, B256, hex::FromHex};
use bankai_types::{
    proofs::{HashingFunctionDto, MmrProofDto},
    utils::mmr::hash_to_leaf,
};

use crate::VerifyError;

pub struct SimpleMmr;

impl SimpleMmr {
    pub fn verify_mmr_proof(proof: &MmrProofDto) -> Result<bool, VerifyError> {
        if proof.elements_count == 0 {
            return Err(VerifyError::InvalidMmrTree);
        }
        if proof.hashing_function != HashingFunctionDto::Keccak {
            return Err(VerifyError::InvalidMmrProof);
        }

        let elements_count = proof.elements_count as usize;
        let element_index = proof.elements_index as usize;
        // println!("elements_count: {}", elements_count);
        // println!("element_index: {}", element_index);
        // if element_index == 0 || element_index > elements_count {
        //     println!("element_index is 0 or greater than elements_count");
        //     return Err(VerifyError::InvalidMmrProof);
        // }

        let expected_peaks_len = validate_mmr_size_and_count_peaks(elements_count)
            .ok_or(VerifyError::InvalidMmrTree)?;
        if proof.peaks.len() != expected_peaks_len {
            println!("expected_peaks_len: {}", expected_peaks_len);
            println!("peaks_len: {}", proof.peaks.len());
            return Err(VerifyError::InvalidMmrTree);
        }

        let (peak_index, peak_height) = get_peak_info(elements_count, element_index)
            .ok_or(VerifyError::InvalidMmrProof)?;
        
        if proof.path.len() != peak_height {
            println!("peak_height: {}", peak_height);
            println!("path_len: {}", proof.path.len());
            return Err(VerifyError::InvalidMmrProof);
        }

        let element_hash_hex = hash_to_leaf(proof.header_hash.clone(), &proof.hashing_function);
        let mut current = parse_b256(&element_hash_hex).ok_or(VerifyError::InvalidHeaderHash)?;
        let mut position = element_index;

        if element_index != elements_count {
            for sib_hex in &proof.path {
                let sibling = parse_b256(sib_hex).ok_or(VerifyError::InvalidMmrProof)?;
                let position_height = height_at(position);
                let next_height = height_at(position + 1);

                if next_height == position_height + 1 {
                    current = hash_pair(&sibling, &current);
                    position += 1;
                } else {
                    current = hash_pair(&current, &sibling);
                    position += 1usize << (position_height + 1);
                }
            }
        }

        let peaks: Vec<B256> = proof
            .peaks
            .iter()
            .map(|h| parse_b256(h).ok_or(VerifyError::InvalidMmrProof))
            .collect::<Result<_, _>>()?;

        if peaks[peak_index] != current {
            println!("peak_index: {}", peak_index);
            println!("current: {}", current);
            println!("peaks[peak_index]: {}", peaks[peak_index]);
            return Err(VerifyError::InvalidMmrProof);
        }

        let bag = bag_peaks_left_to_right(&peaks);
        let root = hash_mmr_root(elements_count as u128, &bag); // encode size and bag deterministically

        let expected_root = parse_b256(&proof.root).ok_or(VerifyError::InvalidMmrRoot)?;
        if root != expected_root {
            println!("root: {}", root);
            println!("expected_root: {}", expected_root);
            return Err(VerifyError::InvalidMmrRoot);
        }

        Ok(true)
    }
}

fn parse_b256(hex: &str) -> Option<B256> {
    B256::from_hex(hex).ok()
}

fn hash_pair(left: &B256, right: &B256) -> B256 {
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(left.as_slice());
    buf[32..].copy_from_slice(right.as_slice());
    keccak256(buf)
}

fn bag_peaks_left_to_right(peaks: &[B256]) -> B256 {
    match peaks.len() {
        0 => B256::ZERO,
        1 => peaks[0],
        _ => {
            let mut acc = *peaks.last().unwrap();
            for i in (0..peaks.len() - 1).rev() {
                acc = hash_pair(&peaks[i], &acc);
            }
            acc
        }
    }
}

fn hash_mmr_root(elements_count: u128, bag: &B256) -> B256 {
    let mut size_be = [0u8; 32];
    size_be[16..].copy_from_slice(&elements_count.to_be_bytes());
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&size_be);
    buf[32..].copy_from_slice(bag.as_slice());
    keccak256(buf)
}

fn validate_mmr_size_and_count_peaks(mut n: usize) -> Option<usize> {
    if n == 0 {
        return None;
    }
    let mut peaks_count = 0usize;
    let mut prev_peak = 0usize;
    while n > 0 {
        let i = bit_length(n);
        if i == 0 {
            return None;
        }
        let peak_tmp = (1usize << i) - 1;
        let peak = if n + 1 <= peak_tmp { (1usize << (i - 1)) - 1 } else { peak_tmp };
        if peak == 0 || peak == prev_peak {
            return None;
        }
        peaks_count += 1;
        n -= peak;
        prev_peak = peak;
    }
    Some(peaks_count)
}

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

fn height_at(mut position: usize) -> usize {
    // postorder index height = trailing ones count - 1
    let mut ones = 0usize;
    while position & 1 == 1 {
        ones += 1;
        position >>= 1;
    }
    ones.saturating_sub(1)
}

fn bit_length(n: usize) -> usize {
    (usize::BITS as usize) - n.leading_zeros() as usize
}


