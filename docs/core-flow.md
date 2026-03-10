# Core Flow

This is the main Bankai SDK workflow:

1. Create `Bankai` with the RPCs you need.
2. Start a batch with `init_batch`.
3. Add proof requests.
4. Call `.execute()` to fetch a `ProofBundle`.
5. Call `verify_batch_proof` to get verified results.

## 1. Configure the SDK

`Bankai::new` takes:

- a Bankai network (`Network::Sepolia` or `Network::Local`)
- an optional Ethereum execution RPC
- an optional Ethereum beacon RPC
- optional OP Stack RPCs keyed by chain name

```rust
use bankai_sdk::{Bankai, Network};

let bankai = Bankai::new(
    Network::Sepolia,
    Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),
    Some("https://sepolia.beacon-api.example.com".to_string()),
    None,
);
```

Use `Bankai::new_with_base_url` if you want to talk to a non-default Bankai API.

`Network::Local` is meant for a fully local setup and defaults Ethereum execution to chain id `31337`. If you want to use a local Bankai API while still fetching Sepolia Ethereum data, keep `Network::Sepolia` and override only the API base URL with `Bankai::new_with_base_url`.

## 2. Initialize a Batch

`init_batch` chooses the Bankai block to anchor the proof bundle and the hashing function used for MMR proofs.

```rust
use bankai_sdk::HashingFunction;

# async fn example(bankai: &bankai_sdk::Bankai) -> Result<(), Box<dyn std::error::Error>> {
let batch = bankai
    .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
    .await?;
# Ok(())
# }
```

- `None` means “use the latest completed Bankai block”.
- `HashingFunction::Keccak` is the normal starting point.

## 3. Add the Data You Need

Keep the batch focused. Ask only for the headers or proofs your application will verify and use.

```rust
use alloy_primitives::Address;

# async fn example(bankai: &bankai_sdk::Bankai) -> Result<(), Box<dyn std::error::Error>> {
let proof_bundle = bankai
    .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
    .await?
    .ethereum_execution_header(9_231_247)
    .ethereum_account(9_231_247, Address::ZERO)
    .execute()
    .await?;
# let _ = proof_bundle;
# Ok(())
# }
```

The batch builder also supports beacon, storage, transaction, receipt, and OP Stack requests.

### Execution Batch

```rust
use alloy_primitives::{Address, FixedBytes, U256};
use bankai_sdk::{Bankai, HashingFunction, Network};

# async fn example(bankai: &Bankai, tx_hash: FixedBytes<32>) -> Result<(), Box<dyn std::error::Error>> {
let batch = bankai
    .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
    .await?
    .ethereum_execution_header(9_231_247)
    .ethereum_account(9_231_247, Address::ZERO)
    .ethereum_storage_slot(9_231_247, Address::ZERO, vec![U256::ZERO])
    .ethereum_tx(tx_hash)
    .ethereum_receipt(tx_hash);
# let _ = batch;
# Ok(())
# }
```

Execution request surface:

- `ethereum_execution_header`
- `ethereum_account`
- `ethereum_storage_slot`
- `ethereum_tx`
- `ethereum_receipt`

### Beacon Batch

```rust
use bankai_sdk::{Bankai, HashingFunction, Network};

# async fn example(bankai: &Bankai) -> Result<(), Box<dyn std::error::Error>> {
let batch = bankai
    .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
    .await?
    .ethereum_beacon_header(5_678_901);
# let _ = batch;
# Ok(())
# }
```

Beacon request surface:

- `ethereum_beacon_header`

### OP Stack Batch

```rust
use alloy_primitives::{Address, FixedBytes, U256};
use bankai_sdk::{Bankai, HashingFunction, Network};

# async fn example(bankai: &Bankai, header_hash: FixedBytes<32>, tx_hash: FixedBytes<32>) -> Result<(), Box<dyn std::error::Error>> {
let batch = bankai
    .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
    .await?
    .op_stack_header("base", 38_381_200)
    .op_stack_latest_header("base")
    .op_stack_header_by_hash("base", header_hash)
    .op_stack_account("base", 38_381_200, Address::ZERO)
    .op_stack_storage_slot("base", 38_381_200, Address::ZERO, vec![U256::ZERO])
    .op_stack_tx("base", tx_hash)
    .op_stack_receipt("base", tx_hash);
# let _ = batch;
# Ok(())
# }
```

OP Stack request surface:

- `op_stack_header`
- `op_stack_latest_header`
- `op_stack_header_by_hash`
- `op_stack_account`
- `op_stack_storage_slot`
- `op_stack_tx`
- `op_stack_receipt`

## How Tx And Receipt Proofs Work

For transaction and receipt requests, the SDK now uses the core proof builder in
`bankai-core` instead of relying on an external trie-proof crate.

The flow is deliberately simple:

1. fetch the target tx to learn the containing block and transaction index
2. fetch the full block transactions or receipts from RPC
3. rebuild the ordered trie locally with Alloy encoding and trie utilities
4. return the proof nodes for the target tx or receipt

For receipts, the fetcher tries `eth_getBlockReceipts` first and falls back to
per-transaction receipt fetches when the RPC does not support block receipts.

This applies to both Ethereum execution chains and OP Stack chains.

## 4. Verify the Bundle

`.execute()` only fetches proof data. The trust boundary is `verify_batch_proof`.

```rust
use bankai_verify::verify_batch_proof;

# fn example(proof_bundle: bankai_sdk::ProofBundle) -> Result<(), Box<dyn std::error::Error>> {
let results = verify_batch_proof(proof_bundle)?;
# let _ = results;
# Ok(())
# }
```

On success:

- the Bankai block proof is valid
- the header inclusion proofs are valid
- the account, storage, transaction, and receipt proofs are valid against those
  headers

## 5. Read Verified Results

`verify_batch_proof` returns `BatchResults`.

```rust
# fn example(results: bankai_types::results::BatchResults) {
for header in &results.evm.execution_header {
    println!("Verified execution block {}", header.number);
}

for account in &results.evm.account {
    println!("Verified account balance {}", account.balance);
}
# }
```

Use the result groups that match your requests:

- `results.evm.*` for Ethereum data
- `results.op_stack.*` for OP Stack data

## Block-Range Trie Root Checks

When you want to validate the proof-building path against live RPC data, run the
root checker binary:

```bash
./scripts/check-trie-roots.sh execution "$EXECUTION_RPC" 10421675 100
./scripts/check-trie-roots.sh op-stack "$BASE_RPC" 38691918 100
```

This recomputes `transactions_root` and `receipts_root` block by block and
fails fast on the first mismatch.

## When To Use the API Client Instead

Use `sdk.api.*` when you need raw endpoint access, custom request DTOs, or proof payloads outside the batch builder flow.

See [API client overview](api-client.md).
