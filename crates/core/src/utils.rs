use cairo_vm_base::vm::cairo_vm::Felt252;
use starknet_crypto::Felt;

pub fn to_felt(value: Felt252) -> Felt {
    Felt::from_bytes_be(&value.to_bytes_be())
}

pub fn to_felt252(value: Felt) -> Felt252 {
    Felt252::from_bytes_be(&value.to_bytes_be())
}
