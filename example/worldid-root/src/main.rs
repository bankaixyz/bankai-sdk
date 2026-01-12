use alloy_primitives::{hex::FromHex, Address, FixedBytes, U256};
use bankai_sdk::{errors::SdkError, Bankai, HashingFunctionDto, Network};
use bankai_verify::verify_batch_proof;
use dotenv::from_filename;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    println!("Initializing Bankai Client...");

    let exec_rpc = std::env::var("EXECUTION_RPC").ok();
    let bankai = Bankai::new(Network::Sepolia, exec_rpc, None);

    println!("Initializing Batch...");
    let batch = bankai
        .init_batch(Network::Sepolia, None, HashingFunctionDto::Keccak)
        .await?;

    println!("Fetching Proof Data...");
    let block_number = 10029096u64; // Sepolia block number
    let contract = Address::from_hex("0xb2EaD588f14e69266d1b87936b75325181377076").unwrap(); // World ID Identity Proxy
    let key_bytes: FixedBytes<32> =
        FixedBytes::from_hex("0x000000000000000000000000000000000000000000000000000000000000012e")
            .unwrap();
    let mpt_key = U256::from_be_bytes(key_bytes.into());

    let wrapper = batch
        .evm_storage_slot(block_number, contract, mpt_key)
        .execute()
        .await?;

    println!("Verifying Proof Data...");
    let verified_data = verify_batch_proof(wrapper)?;

    println!(
        "Latest World ID Root: {:?}",
        verified_data.evm.storage_slot[0]
    );

    Ok(())
}
