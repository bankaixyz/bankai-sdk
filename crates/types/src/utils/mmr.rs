use alloy_primitives::{
    B256,
    hex::{FromHex, ToHexExt},
    keccak256,
};
use starknet_crypto::{Felt, poseidon_hash};

use crate::api::HashingFunctionDto;

pub fn hash_to_leaf(hash: String, hashing_function: &HashingFunctionDto) -> String {
    let hash = B256::from_hex(hash).unwrap();
    match hashing_function {
        HashingFunctionDto::Keccak => {
            let hashed_root = keccak256(hash.as_slice());
            format!("0x{}", hashed_root.encode_hex())
        }
        HashingFunctionDto::Poseidon => {
            let root_bytes = hash.as_slice();
            let high = Felt::from_bytes_be_slice(&root_bytes[0..16]);
            let low = Felt::from_bytes_be_slice(&root_bytes[16..32]);

            let hashed_root = poseidon_hash(low, high);
            hashed_root.to_fixed_hex_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex::FromHex;

    #[test]
    fn test_hash_to_leaf_keccak() {
        let input =
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let expected =
            "0xcae36a6a44328f3fb063df12b0cf3fa225a3c6dbdd6acef0f6e619d33890cf24".to_string();
        let result = hash_to_leaf(input, &HashingFunctionDto::Keccak);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_hash_to_leaf_poseidon() {
        let input =
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let expected =
            "0x05206aa252b669b3d3348eede13d91a5002293e2da9f3ca4ee905dd2578793b9".to_string();
        let result = hash_to_leaf(input, &HashingFunctionDto::Poseidon);
        assert_eq!(result, expected);
    }
}
