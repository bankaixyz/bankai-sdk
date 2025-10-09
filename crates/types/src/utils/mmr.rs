use alloy_primitives::{hex::{FromHex, ToHexExt}, keccak256, B256};
use starknet_crypto::{poseidon_hash, Felt};

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
            format!("0x{}", hashed_root.to_hex_string())
        }
    }
}