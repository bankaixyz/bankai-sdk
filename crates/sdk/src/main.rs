use bankai_sdk::{errors::SdkError, Bankai};
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

    if exec_rpc.is_some() {
        let exec = bankai.evm.execution()?;
        let proof = exec
            .header(
                exec_block_number,
                HashingFunctionDto::Keccak,
                bankai_block_number,
            )
            .await?;
        let header = bankai.verify.evm_execution_header(&proof).await?;
        println!("Verfied Execution header: {:?}", header);
    }

    if beacon_rpc.is_some() {
        let beacon = bankai.evm.beacon()?;
        let proof = beacon
            .header(beacon_slot, HashingFunctionDto::Keccak, bankai_block_number)
            .await?;
        let header = bankai.verify.evm_beacon_header(&proof).await?;
        println!("Verfied Beacon header: {:?}", header);
    }

    Ok(())
}
