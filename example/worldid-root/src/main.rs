use alloy_primitives::{hex::FromHex, Address, FixedBytes, U256};
use bankai_sdk::{errors::SdkError, Bankai, HashingFunction, Network};
use bankai_verify::verify_batch_proof;
use dotenv::from_filename;

const BASE_BLOCK_NUMBER: u64 = 38764232;
const BASE_CONTRACT_ADDRESS: &str = "0x2A7f20A455B35ea3cfF416F71dDB30E0eDF5c9fE";
// const BASE_TX_HASH: &str = "0xa6fc949201f67c79f9f627349a36c19d7760427e3196a57deaf0f270874690c3"; old one with err
const BASE_TX_HASH: &str = "0x2932eb79df0313298bb2d49edff60855fdd76281006edc6c48f59cfe1f390cc9";
const ETH_BLOCK_NUMBER: u64 = 10_421_675;
const ETH_ACCOUNT_ADDRESS: &str = "0x0000006916a87b82333f4245046623b23794c65c";

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    from_filename(".env").ok();

    let base_rpc =
        std::env::var("BASE_RPC").map_err(|_| SdkError::NotConfigured("BASE_RPC".to_string()))?;

    let mut op_rpc = std::collections::BTreeMap::new();
    op_rpc.insert("base".to_string(), base_rpc);

    let execution_rpc = std::env::var("EXECUTION_RPC")
        .map_err(|_| SdkError::NotConfigured("EXECUTION_RPC".to_string()))?;

    let bankai = Bankai::new(Network::Local, Some(execution_rpc), None, Some(op_rpc));
    let contract = Address::from_hex(BASE_CONTRACT_ADDRESS).unwrap();
    let execution_account = Address::from_hex(ETH_ACCOUNT_ADDRESS).unwrap();
    let tx_hash = FixedBytes::from_hex(BASE_TX_HASH).unwrap();

    let op_stack_proof_bundle = bankai
        .init_batch(None, HashingFunction::Keccak)
        .await?
        .op_stack_account(
            "base",
            BASE_BLOCK_NUMBER,
            Address::from_hex("0xcF93D9de9965B960769aa9B28164D571cBbCE39C").unwrap(),
        )
        .op_stack_storage_slot("base", BASE_BLOCK_NUMBER, contract, vec![U256::ZERO])
        .op_stack_tx("base", tx_hash)
        .op_stack_receipt("base", tx_hash)
        .execute()
        .await?;

    let tx_index = op_stack_proof_bundle
        .op_stack_proofs
        .as_ref()
        .unwrap()
        .tx_proof[0]
        .tx_index;

    let op_stack_results = verify_batch_proof(op_stack_proof_bundle)?;

    let op_stack_header = &op_stack_results.op_stack.header[0];
    let op_stack_account = &op_stack_results.op_stack.account[0];
    let op_stack_storage = &op_stack_results.op_stack.storage_slot[0];
    let (op_stack_slot_key, op_stack_slot_value) = op_stack_storage.slots[0];
    let tx = &op_stack_results.op_stack.tx[0];
    let receipt = &op_stack_results.op_stack.receipt[0];

    println!("Verified Base OP Stack block {}", op_stack_header.number);
    println!(
        "Verified Base account {} balance {} at block {}",
        op_stack_account.address, op_stack_account.account.balance, op_stack_account.block.block_number
    );
    println!(
        "Verified Base storage slot {} at block {}: {op_stack_slot_key} = {op_stack_slot_value}",
        op_stack_storage.address, op_stack_storage.block.block_number
    );
    println!(
        "Verified tx {} at block {} index {} with type {:?}",
        tx.tx_hash,
        tx.block.block_number,
        tx.tx_index,
        tx.tx_type()
    );
    println!(
        "Verified receipt {} at block {} status={} cumulative_gas_used={}",
        receipt.tx_hash,
        receipt.block.block_number,
        receipt.receipt.status(),
        receipt.receipt.cumulative_gas_used()
    );

    let execution_proof_bundle = bankai
        .init_batch(None, HashingFunction::Keccak)
        .await?
        .ethereum_execution_header(ETH_BLOCK_NUMBER)
        .ethereum_account(ETH_BLOCK_NUMBER, execution_account)
        .ethereum_storage_slot(ETH_BLOCK_NUMBER, execution_account, vec![U256::ZERO])
        .execute()
        .await?;

    let execution_results = verify_batch_proof(execution_proof_bundle)?;

    let execution_header = &execution_results.evm.execution_header[0];
    let execution_account_result = &execution_results.evm.account[0];
    let execution_storage = &execution_results.evm.storage_slot[0];
    let (execution_slot_key, execution_slot_value) = execution_storage.slots[0];

    println!(
        "Verified Sepolia execution block {}",
        execution_header.number
    );
    println!(
        "Verified execution account {} balance {} at block {}",
        execution_account_result.address,
        execution_account_result.account.balance,
        execution_account_result.block.block_number
    );
    println!(
        "Verified execution storage slot {} at block {}: {execution_slot_key} = {execution_slot_value}",
        execution_storage.address, execution_storage.block.block_number
    );

    Ok(())
}
