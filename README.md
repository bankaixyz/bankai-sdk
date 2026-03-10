# Bankai SDK

Trustless blockchain data access through Bankai proof bundles.

The SDK is built around one flow:

1. Configure `Bankai` with the RPCs you want to read from.
2. Create a batch, execute it, and get a `ProofBundle`.
3. Verify the bundle with `bankai-verify` and read the verified results.

## Current Scope

- Ethereum Sepolia is the primary network surface today.
- The batch builder supports Ethereum and OP Stack requests.
- Transaction and receipt proofs are built locally in `bankai-core` by
  reconstructing the ordered trie from RPC-fetched block transactions or
  receipts.
- The low-level API client is available when you need raw endpoint access.

## Setup

Add the crates and the required `ethereum_hashing` patch:

```toml
[dependencies]
bankai-sdk = "0.1"
bankai-verify = "0.1"
bankai-types = "0.1"
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e", default-features = false, features = ["portable"] }

[patch.crates-io]
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e" }
```

## Feature Flags

`bankai-types` is split so consumers can keep dependencies narrow:

- `results` gives you the verified output types with the lightest surface.
- `inputs` gives you `ProofBundle` and verifier input types without pulling in the API/OpenAPI layer.
- `api` gives you the raw Bankai API DTOs and request/response types.

That means zkVM or verifier-focused consumers can depend on only the pieces they need instead of compiling the full API stack.

## Quickstart

```rust
use alloy_primitives::Address;
use bankai_sdk::{Bankai, HashingFunction, Network};
use bankai_verify::verify_batch_proof;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bankai = Bankai::new(
        Network::Sepolia,
        Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),
        Some("https://sepolia.beacon-api.example.com".to_string()),
        None,
    );

    let proof_bundle = bankai
        .init_batch(Network::Sepolia, None, HashingFunction::Keccak)
        .await?
        .ethereum_execution_header(9_231_247)
        .ethereum_account(9_231_247, Address::ZERO)
        .execute()
        .await?;

    let results = verify_batch_proof(proof_bundle)?;

    let header = &results.evm.execution_header[0];
    let account = &results.evm.account[0];

    println!("Verified block {}", header.number);
    println!("Verified balance {}", account.balance);

    Ok(())
}
```

## More Docs

- [Core flow](docs/core-flow.md)
- [OP Stack integration](docs/op-stack.md)
- [API client overview](docs/api-client.md)
- [World ID OP Stack example](example/worldid-root/README.md)

## Proof Types

Transaction and receipt proofs no longer depend on an external trie-proof crate.
`bankai-core` now owns that logic for both Ethereum execution chains and OP Stack
chains:

- fetch the target transaction to resolve its block and index
- fetch all block transactions or receipts from RPC
- rebuild the ordered trie with Alloy encoders and trie utilities
- return the proof nodes plus the encoded tx or receipt payload

On OP Stack, this path uses `op-alloy`, so deposit transactions and OP-specific
receipt fields are handled correctly.

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

Execution proof requests:

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

Beacon proof requests:

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
    .op_stack_account("base", 38_381_200, Address::ZERO)
    .op_stack_storage_slot("base", 38_381_200, Address::ZERO, vec![U256::ZERO])
    .op_stack_tx("base", tx_hash)
    .op_stack_receipt("base", tx_hash);
# let _ = batch;
# Ok(())
# }
```

OP Stack proof requests:

- `op_stack_header`
- `op_stack_latest_header`
- `op_stack_header_by_hash`
- `op_stack_account`
- `op_stack_storage_slot`
- `op_stack_tx`
- `op_stack_receipt`

Use `Network::Local` when you are working against a fully local Bankai deployment. It defaults Ethereum execution to chain id `31337`.

If you want to point at a local Bankai API while still targeting Sepolia Ethereum data, prefer `Bankai::new_with_base_url(Network::Sepolia, "http://localhost:8080".to_string(), ...)`.
