# Bankai SDK

Trustless blockchain data access through Bankai proof bundles.

The SDK is built around one flow:

1. Configure `Bankai` with the RPCs you want to read from.
2. Create a batch, execute it, and get a `ProofBundle`.
3. Verify the bundle with `bankai-verify` and read the verified results.

## Current Scope

- Ethereum Sepolia is the primary network surface today.
- The batch builder supports Ethereum and OP Stack requests.
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

Use `Network::Local` when you are working against a fully local Bankai deployment. It defaults Ethereum execution to chain id `31337`.

If you want to point at a local Bankai API while still targeting Sepolia Ethereum data, prefer `Bankai::new_with_base_url(Network::Sepolia, "http://localhost:8080".to_string(), ...)`.
