use alloy_primitives::{hex::FromHex, Address};
use bankai_sdk::{errors::SdkError, Bankai, HashingFunction, Network};
use bankai_verify::verify_batch_proof;
use dotenv::from_filename;

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    let base_rpc =
        std::env::var("BASE_RPC").map_err(|_| SdkError::NotConfigured("BASE_RPC".to_string()))?;
    let mut op_rpc = std::collections::BTreeMap::new();
    op_rpc.insert("base".to_string(), base_rpc);

    let bankai = Bankai::new(Network::Local, None, None, Some(op_rpc));

    let proof_bundle = bankai
        .init_batch(Network::Local, None, HashingFunction::Keccak)
        .await?
        .op_stack_account(
            "base",
            38381200,
            Address::from_hex("0xcF93D9de9965B960769aa9B28164D571cBbCE39C").unwrap(),
        )
        .execute()
        .await?;

    let results = verify_batch_proof(proof_bundle)?;
    let header = &results.op_stack.header[0];
    let account = &results.op_stack.account[0];

    println!("Verified OP Stack block {}", header.number);
    println!("Verified balance {}", account.balance);

    Ok(())
}
