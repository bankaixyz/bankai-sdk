use alloy_primitives::{hex::FromHex, Address};
use bankai_sdk::{errors::SdkError, Bankai, HashingFunction, Network};
use bankai_verify::verify_batch_proof;
use dotenv::from_filename;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    println!("Initializing Bankai Client...");

    let exec_rpc = std::env::var("EXECUTION_RPC").ok();
    let base_rpc = std::env::var("BASE_RPC").ok();
    let mut op_rpc = std::collections::BTreeMap::new();
    op_rpc.insert("base".to_string(), base_rpc.unwrap());
    let bankai = Bankai::new(Network::Local, exec_rpc, None, Some(op_rpc));

    println!("Initializing Batch...");
    let batch = bankai
        .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
        .await?;

    let proof_bundle = batch
        .op_stack_account(
            "base",
            38381200,
            Address::from_hex("0xcF93D9de9965B960769aa9B28164D571cBbCE39C").unwrap(),
        )
        .execute()
        .await?;

    // println!("Fetching Proof Data...");
    // let block_number = 10029096u64; // Sepolia block number
    // let contract = Address::from_hex("0xb2EaD588f14e69266d1b87936b75325181377076").unwrap(); // World ID Identity Proxy
    // let key_bytes: FixedBytes<32> =
    //     FixedBytes::from_hex("0x000000000000000000000000000000000000000000000000000000000000012e")
    //         .unwrap();
    // let mpt_key = U256::from_be_bytes(key_bytes.into());

    // let wrapper = batch
    //     .ethereum_storage_slot(block_number, contract, vec![mpt_key])
    //     .execute()
    //     .await?;

    // println!("Verifying Proof Data...");
    let verified_data = verify_batch_proof(proof_bundle)?;
    println!("Verified data: {:?}", verified_data);
    // let (slot_key, slot_value) = &verified_data.evm.storage_slot[0][0];
    // println!("Latest World ID Root: slot {slot_key} = {slot_value:?}");

    Ok(())
}
