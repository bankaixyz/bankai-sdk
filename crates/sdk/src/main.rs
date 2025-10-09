use alloy_primitives::hex::{FromHex, ToHexExt};
use bankai_sdk::api_client::ApiClient;
use bankai_sdk::fetch::stwo::fetch_block_proof;
use bankai_sdk::verify::mmr::BankaiMmr;
use bankai_sdk::verify::stwo::verify_stwo_proof;
use bankai_types::api::{HashingFunctionDto, MmrProofRequestDto};
use bankai_types::block::BankaiBlock;
use cairo_air::utils::get_verification_output;
use cairo_air::{CairoProof, PreProcessedTraceVariant};
use stwo::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};

#[tokio::main]
async fn main() {
    let client = ApiClient::new("https://sepolia.api.bankai.xyz".to_string());
    let bankai_block_number = 11261;
    let stwo_proof = fetch_block_proof(&client, bankai_block_number)
        .await
        .unwrap();

    let block = verify_stwo_proof(&stwo_proof).unwrap();

    println!("Bankai Block Verified: {:?}", block);

    let mmr_proof_keccak = client
        .get_mmr_proof(&MmrProofRequestDto {
            network_id: 1,
            block_number: bankai_block_number,
            hashing_function: HashingFunctionDto::Keccak,
            header_hash: "0x396ab18184a742b0252eff9c83b3bb9c48f05abec530813cc24f8a18dcb47ac8"
                .to_string(),
        })
        .await
        .unwrap();

    let mmr_proof_poseidon = client
        .get_mmr_proof(&MmrProofRequestDto {
            network_id: 1,
            block_number: bankai_block_number,
            hashing_function: HashingFunctionDto::Poseidon,
            header_hash: "0x396ab18184a742b0252eff9c83b3bb9c48f05abec530813cc24f8a18dcb47ac8"
                .to_string(),
        })
        .await
        .unwrap();

    match mmr_proof_keccak.hashing_function {
        HashingFunctionDto::Keccak => {
            assert_eq!(
                mmr_proof_keccak.root,
                format!("0x{}", block.execution.mmr_root_keccak.encode_hex())
            );
        }
        HashingFunctionDto::Poseidon => {
            assert_eq!(
                mmr_proof_keccak.root,
                format!("0x{}", block.execution.mmr_root_poseidon.encode_hex())
            );
        }
    }

    let mmr_proof_verified =
        BankaiMmr::verify_proof(mmr_proof_keccak.clone().hashing_function, mmr_proof_keccak)
            .await
            .unwrap();

    println!("Mmr Proof valid: {:?}", mmr_proof_verified);
    // println!("{:?}", proof);
}
