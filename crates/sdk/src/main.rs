use bankai_sdk::api_client::ApiClient;
use bankai_types::block::BankaiBlock;
use cairo_air::utils::get_verification_output;
use cairo_air::{CairoProof, PreProcessedTraceVariant};
use stwo::core::vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher};


#[tokio::main]
async fn main() {
    let client = ApiClient::new("https://sepolia.api.bankai.xyz".to_string());
    let proof = client.get_zk_proof(11261).await.unwrap();

    let stwo_proof: CairoProof<Blake2sMerkleHasher> = serde_json::from_value(proof.proof).unwrap();

    let verification_output = get_verification_output(&stwo_proof.claim.public_data.public_memory);

    let block = BankaiBlock::from_verication_output(&verification_output);
    
    println!("{:?}", block);

    cairo_air::verifier::verify_cairo::<Blake2sMerkleChannel>(
        stwo_proof,
        PreProcessedTraceVariant::CanonicalWithoutPedersen,
    ).unwrap();
    
    // println!("{:?}", proof);
}