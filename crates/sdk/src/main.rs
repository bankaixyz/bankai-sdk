use alloy_primitives::{hex::FromHex, Address, FixedBytes};
use bankai_sdk::{errors::SdkError, Bankai, Network};
use bankai_verify::verify_batch_proof;

use bankai_types::proofs::HashingFunctionDto;
use dotenv::from_filename;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    let exec_rpc = std::env::var("EXECUTION_RPC").ok();
    let beacon_rpc = std::env::var("BEACON_RPC").ok();
    let bankai = Bankai::new(Network::Sepolia, exec_rpc.clone(), beacon_rpc.clone());

    let proof_batch = bankai
        .init_batch(Network::Sepolia, None, HashingFunctionDto::Poseidon)
        .await?
        .evm_beacon_header(8551383)
        .evm_execution_header(9231247)
        .evm_account(9231247, Address::ZERO)
        .evm_tx(
            FixedBytes::from_hex(
                "0x501b7c72c1e5f14f02e1a58a7264e18f5e26a793d42e4e802544e6629764f58c",
            )
            .unwrap(),
        )
        .evm_tx(
            FixedBytes::from_hex(
                "0xd7e25cbf8ff63e3d9e4fa1e9783afae248a50df836f2cd853f89440f4c76891d",
            )
            .unwrap(),
        )
        .evm_tx(
            FixedBytes::from_hex(
                "0x0c859ef15b3f7ee56ae691c285f23650b864267e7813d746f75409a142e03622",
            )
            .unwrap(),
        )
        .execute()
        .await?;

    let valid_data = verify_batch_proof(proof_batch)?;
    println!("valid data: {valid_data:#?}");

    Ok(())
}
