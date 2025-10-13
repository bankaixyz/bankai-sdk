use alloy_primitives::{hex::FromHex, Address, FixedBytes};
use bankai_sdk::{errors::SdkError, verify::batch::verify_batch_proof, Bankai};
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
    let proof_batch = bankai
        .init_batch(bankai_block_number, HashingFunctionDto::Keccak)
        .evm_beacon_header(0, beacon_slot) // beacon network id 0
        .evm_execution_header(1, exec_block_number) // execution network id 1
        .evm_account(1, exec_block_number, Address::ZERO)
        .evm_tx(
            1,
            FixedBytes::from_hex(
                "0x501b7c72c1e5f14f02e1a58a7264e18f5e26a793d42e4e802544e6629764f58c",
            )
            .unwrap(),
        )
        .evm_tx(
            1,
            FixedBytes::from_hex(
                "0xd7e25cbf8ff63e3d9e4fa1e9783afae248a50df836f2cd853f89440f4c76891d",
            )
            .unwrap(),
        )
        .evm_tx(
            1,
            FixedBytes::from_hex(
                "0x0c859ef15b3f7ee56ae691c285f23650b864267e7813d746f75409a142e03622",
            )
            .unwrap(),
        )
        .execute()
        .await?;

    let valid_data = verify_batch_proof(&proof_batch).await?;
    println!("valid data: {valid_data:#?}");

    Ok(())
}
