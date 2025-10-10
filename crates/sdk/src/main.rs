use bankai_sdk::fetch::clients::bankai_api::ApiClient;
use bankai_sdk::fetch::evm::beacon::BeaconChainFetcher;
use bankai_sdk::verify::evm::beacon::BeaconVerifier;
use bankai_types::api::HashingFunctionDto;
use dotenv::from_filename;

#[tokio::main]
async fn main() {
    from_filename(".env").ok();
    let client = ApiClient::new("https://sepolia.api.bankai.xyz".to_string());
    let bankai_block_number = 11261;
    let exec_block_number = 8292000;

    let exex_rpc = std::env::var("EXECUTION_RPC").expect("EXECUTION_RPC must be set");
    let beacon_rpc = std::env::var("BEACON_RPC").expect("BEACON_RPC must be set");

    let proof_fetcher = BeaconChainFetcher::new(client, beacon_rpc, 0);
    let header_proof = proof_fetcher
        .header(
            exec_block_number,
            HashingFunctionDto::Keccak,
            bankai_block_number,
        )
        .await
        .unwrap();
    let beacon_header = BeaconVerifier::verify_header_proof(&header_proof)
        .await
        .unwrap();
    println!("Beacon header: {beacon_header:?}");
    // let header = ExecutionVerifier::verify_header_proof(&header_proof).await.unwrap();
    // println!("Header: {:?}", header);

    // let proof_fetcher = ExecutionChainFetcher::new(client, "https://quick-crimson-needle.ethereum-sepolia.quiknode.pro/5da9fed24a0876297c00a0d358d33a324455edcb".to_string(), 1);
    // let header_proof = proof_fetcher
    //     .header(
    //         exec_block_number,
    //         HashingFunctionDto::Keccak,
    //         bankai_block_number,
    //     )
    //     .await
    //     .unwrap();
    // let header = ExecutionVerifier::verify_header_proof(&header_proof)
    //     .await
    //     .unwrap();
    // println!("Header: {header:?}");
}
