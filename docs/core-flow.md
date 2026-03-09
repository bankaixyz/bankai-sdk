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
- the Merkle proofs are valid against those headers

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

## When To Use the API Client Instead

Use `sdk.api.*` when you need raw endpoint access, custom request DTOs, or proof payloads outside the batch builder flow.

See [API client overview](api-client.md).
