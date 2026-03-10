use alloy_primitives::{hex::FromHex, Address, FixedBytes, U256};
use bankai_sdk::{errors::SdkError, Bankai, HashingFunction, Network};
use bankai_verify::verify_batch_proof;
use dotenv::from_filename;

const BLOCK_NUMBER: u64 = 38_691_918;
const CONTRACT_ADDRESS: &str = "0x2A7f20A455B35ea3cfF416F71dDB30E0eDF5c9fE";
const TX_HASH: &str = "0xa6fc949201f67c79f9f627349a36c19d7760427e3196a57deaf0f270874690c3";

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    let base_rpc =
        std::env::var("BASE_RPC").map_err(|_| SdkError::NotConfigured("BASE_RPC".to_string()))?;

    let mut op_rpc = std::collections::BTreeMap::new();
    op_rpc.insert("base".to_string(), base_rpc);

    let bankai = Bankai::new(Network::Local, None, None, Some(op_rpc));
    let contract = Address::from_hex(CONTRACT_ADDRESS).unwrap();
    // let tx_hash = FixedBytes::from_hex(TX_HASH).unwrap();

    let proof_bundle = bankai
        .init_batch(Network::Local, None, HashingFunction::Keccak)
        .await?
        .op_stack_account("base", BLOCK_NUMBER, Address::from_hex("0xcF93D9de9965B960769aa9B28164D571cBbCE39C").unwrap())
        .op_stack_storage_slot("base", BLOCK_NUMBER, contract, vec![U256::ZERO])
        // .op_stack_tx("base", tx_hash)
        // .op_stack_receipt("base", tx_hash)
        .execute()
        .await?;

    // let tx_index = proof_bundle.op_stack_proofs.as_ref().unwrap().tx_proof[0].tx_index;

    let results = verify_batch_proof(proof_bundle)?;

    let header = &results.op_stack.header[0];
    let account = &results.op_stack.account[0];
    let (slot_key, slot_value) = results.op_stack.storage_slot[0][0];
    // let tx = &results.op_stack.tx[0];
    // let receipt = &results.op_stack.receipt[0];

    println!("Verified Base OP Stack block {}", header.number);
    println!("Verified account {} balance {}", contract, account.balance);
    println!("Verified storage slot {slot_key} = {slot_value}");
    // println!(
    //     "Verified tx {} at index {} with type {:?}",
    //     TX_HASH,
    //     tx_index,
    //     tx.tx_type()
    // );
    // println!(
    //     "Verified receipt status={} cumulative_gas_used={}",
    //     receipt.status(),
    //     receipt.cumulative_gas_used()
    // );

    Ok(())
}
