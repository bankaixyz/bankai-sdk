use bankai_sdk::fetch::clients::bankai_api::ApiClient;
use bankai_sdk::fetch::evm::execution::ExecutionChainFetcher;
use bankai_sdk::verify::evm::execution::ExecutionVerifier;
use bankai_types::api::HashingFunctionDto;

#[tokio::main]
async fn main() {
    let client = ApiClient::new("https://sepolia.api.bankai.xyz".to_string());
    let bankai_block_number = 11261;
    let exec_block_number = 9241218;

    let proof_fetcher = ExecutionChainFetcher::new(client, "https://quick-crimson-needle.ethereum-sepolia.quiknode.pro/5da9fed24a0876297c00a0d358d33a324455edcb".to_string(), 1);
    let header_proof = proof_fetcher
        .header(
            exec_block_number,
            HashingFunctionDto::Keccak,
            bankai_block_number,
        )
        .await
        .unwrap();
    let header = ExecutionVerifier::verify_header_proof(&header_proof)
        .await
        .unwrap();
    println!("Header: {header:?}");
}
