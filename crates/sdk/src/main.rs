use alloy_primitives::Address;
use bankai_sdk::{errors::SdkError, verify::batch::verify_wrapper, Bankai};
use bankai_types::api::proofs::HashingFunctionDto;
use dotenv::from_filename;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    let exec_rpc = std::env::var("EXECUTION_RPC").ok();
    let beacon_rpc = std::env::var("BEACON_RPC").ok();
    let bankai = Bankai::new(exec_rpc.clone(), beacon_rpc.clone());

    let bankai_block_number = 11260u64;
    let exec_block_number = 9231247u64;
    let beacon_slot = 8551383u64;

    // Build a single batch containing: beacon header, execution header, and account proof
    let proof_wrapper = bankai
        .init_batch(bankai_block_number, HashingFunctionDto::Keccak)
        .evm_beacon_header(0, beacon_slot) // beacon network id 0
        .evm_execution_header(1, exec_block_number) // execution network id 1
        .evm_account(1, exec_block_number, Address::ZERO)
        .execute()
        .await?;

    let valid_data = verify_wrapper(&proof_wrapper).await?;
    println!("valid data: {:#?}", valid_data);

    Ok(())
}
