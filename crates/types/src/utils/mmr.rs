extern crate alloc;
use alloy_primitives::FixedBytes;
use alloy_primitives::keccak256;
use starknet_crypto::{poseidon_hash, Felt};

use crate::proofs::HashingFunctionDto;

pub fn hash_to_leaf(hash: FixedBytes<32>, hashing_function: &HashingFunctionDto) -> FixedBytes<32> {
    match hashing_function {
        HashingFunctionDto::Keccak => {
            let hashed_root = keccak256(hash.as_slice());
            hashed_root
        }
        HashingFunctionDto::Poseidon => {
            let root_bytes = hash.as_slice();
            let high = Felt::from_bytes_be_slice(&root_bytes[0..16]);
            let low = Felt::from_bytes_be_slice(&root_bytes[16..32]);

            let hashed_root = poseidon_hash(low, high);
            FixedBytes::from_slice(hashed_root.to_bytes_be().as_slice())
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex::FromHex;

    use super::*;

    #[test]
    fn test_hash_to_leaf_keccak() {
        let input =
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let expected =
            "0xcae36a6a44328f3fb063df12b0cf3fa225a3c6dbdd6acef0f6e619d33890cf24".to_string();
        let result = hash_to_leaf(FixedBytes::from_hex(input).unwrap(), &HashingFunctionDto::Keccak);
        assert_eq!(result, FixedBytes::from_hex(expected).unwrap());
    }

    #[test]
    fn test_hash_to_leaf_poseidon() {
        let input =
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let expected =
            "0x05206aa252b669b3d3348eede13d91a5002293e2da9f3ca4ee905dd2578793b9".to_string();
        let result = hash_to_leaf(FixedBytes::from_hex(input).unwrap(), &HashingFunctionDto::Poseidon);
        assert_eq!(result, FixedBytes::from_hex(expected).unwrap());
    }
}
