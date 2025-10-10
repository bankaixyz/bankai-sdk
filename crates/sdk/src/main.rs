use bankai_sdk::{Bankai, errors::SdkError};
use bankai_types::api::proofs::HashingFunctionDto;
use dotenv::from_filename;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    let exec_rpc = std::env::var("EXECUTION_RPC").ok();
    let beacon_rpc = std::env::var("BEACON_RPC").ok();

    let mut builder = Bankai::builder().with_api_base("https://sepolia.api.bankai.xyz".to_string());
    if let Some(rpc) = exec_rpc.clone() { builder = builder.with_evm_execution(rpc); }
    if let Some(rpc) = beacon_rpc.clone() { builder = builder.with_evm_beacon(rpc); }
    let bankai = builder.build();

    let bankai_block_number = 11261u64;
    let exec_block_number = 8_292_000u64;
    let beacon_slot = 1_234_567u64;

    if let Some(exec) = bankai.evm.execution.as_ref() {
        let proof = exec
            .header(exec_block_number, HashingFunctionDto::Keccak, bankai_block_number)
            .await?;
        let _header = bankai.verify.evm_execution_header(&proof).await?;
        println!("Execution header verified (hash bound via MMR)");
    } else {
        println!("EXECUTION_RPC not set; skipping execution proof demo");
    }

    if let Some(beacon) = bankai.evm.beacon.as_ref() {
        let proof = beacon
            .header(beacon_slot, HashingFunctionDto::Keccak, bankai_block_number)
            .await?;
        let _header = bankai.verify.evm_beacon_header(&proof).await?;
        println!("Beacon header verified (slot bound via MMR)");
    } else {
        println!("BEACON_RPC not set; skipping beacon proof demo");
    }

    Ok(())
}
