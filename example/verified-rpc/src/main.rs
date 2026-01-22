use std::env;

use alloy_primitives::hex;
use alloy_provider::{ProviderBuilder};
use bankai_example_verified_rpc::{VerifiedProvider, VerifiedRpcClient};
use bankai_sdk::Network;

#[cfg(feature = "native")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let rpc_url = env::var("RPC_URL")
        .expect("RPC_URL must be set to an execution JSON-RPC endpoint");
    let block_number: u64 = env::var("BLOCK_NUMBER")
        .expect("BLOCK_NUMBER must be set to a historical block height")
        .parse()?;
    let bankai_block_number = env::var("BANKAI_BLOCK_NUMBER")
        .ok()
        .map(|value| value.parse::<u64>())
        .transpose()?;
    let bankai_api_base = env::var("BANKAI_API_BASE").ok();
    let mode = env::var("MODE").unwrap_or_else(|_| "provider".to_string());

    let verified = match mode.as_str() {
        "provider" => {
            let url: reqwest::Url = rpc_url.parse()?;
            let provider = ProviderBuilder::new().connect_http(url);
            let verified_provider = VerifiedProvider::new(Network::Sepolia, provider, bankai_api_base);
            verified_provider
                .get_block_by_number_verified(block_number, bankai_block_number)
                .await?
        }
        "client" => {
            let client = VerifiedRpcClient::new(Network::Sepolia, rpc_url, bankai_api_base);
            client
                .get_block_by_number_verified(block_number, bankai_block_number)
                .await?
        }
        _ => {
            return Err(format!(
                "MODE must be 'provider' or 'client' (got '{mode}')"
            )
            .into());
        }
    };

    println!("Verified block {}", block_number);
    println!("Mode: {}", mode);
    println!("Header hash: 0x{}", hex::encode(verified.header_hash));
    println!("MMR root: 0x{}", hex::encode(verified.mmr_root));
    println!("Bankai block: {}", verified.bankai_block_number);

    Ok(())
}

#[cfg(not(feature = "native"))]
fn main() {
    eprintln!("This demo binary requires the `native` feature.");
}
