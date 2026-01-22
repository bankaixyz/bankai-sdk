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
    let url: reqwest::Url = rpc_url.parse()?;
    let provider = ProviderBuilder::new().connect_http(url);
    let verified = VerifiedProvider::new(Network::Sepolia, provider);

    let latest = verified.get_block_number().await?;
    let header = verified
        .get_block_by_number_verified(latest - 100, None)
        .await?;

    println!("Verified header hash: 0x{}", hex::encode(header.header_hash));

    Ok(())
}

#[cfg(not(feature = "native"))]
fn main() {
    eprintln!("This demo binary requires the `native` feature.");
}
