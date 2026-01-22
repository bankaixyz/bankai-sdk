use std::env;

use alloy_primitives::hex;
use alloy_provider::{Provider, ProviderBuilder};
use bankai_example_verified_rpc::VerifiedProvider;
use bankai_sdk::Network;

#[cfg(feature = "native")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let rpc_url = env::var("RPC_URL")
        .expect("RPC_URL must be set to an execution JSON-RPC endpoint");
    let bankai_api_base = env::var("BANKAI_API_BASE").ok();

    let url: reqwest::Url = rpc_url.parse()?;
    let provider = ProviderBuilder::new().connect_http(url);
    let verified = VerifiedProvider::new(Network::Sepolia, provider, bankai_api_base);

    let latest = verified.get_block_number().await?;
    let header = verified
        .get_block_by_number_verified(latest, None)
        .await?;

    println!("Latest block: {}", latest);
    println!("Verified header hash: 0x{}", hex::encode(header.header_hash));
    println!("MMR root: 0x{}", hex::encode(header.mmr_root));
    println!("Bankai block: {}", header.bankai_block_number);

    Ok(())
}

#[cfg(not(feature = "native"))]
fn main() {
    eprintln!("This demo binary requires the `native` feature.");
}
