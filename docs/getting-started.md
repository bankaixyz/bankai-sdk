# Getting Started

This is the fastest way to understand how Bankai fits together and what you need to run your first proof flow.

## The Mental Model

The Bankai SDK has two main jobs:

1. `bankai-sdk` fetches data and assembles a `ProofBundle`
2. `bankai-verify` checks that bundle and returns trusted results

Most users should start with the batch builder, not the low-level API client.

## What You Need

Depending on what you want to verify, you may need:

- an Ethereum execution RPC
- an Ethereum beacon RPC
- a map of OP Stack RPCs keyed by chain name, such as `"base"` or `"worldchain"`

You do not need to configure every RPC up front. Only provide the ones needed for the proofs you plan to request.

## Configure The SDK

`Network::Sepolia` is the normal starting point:

```rust
use bankai_sdk::{Bankai, Network};

let bankai = Bankai::new(
    Network::Sepolia,
    Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),
    Some("https://sepolia.beacon-api.example.com".to_string()),
    None,
);
```

For normal SDK usage, start here and add only the RPCs you need for the proofs you want to build.

`Network::Local` and custom API base URLs are mainly for development and local Bankai deployments, so they are intentionally left out of the main onboarding path.

## First Steps

This is the smallest useful end-to-end flow:

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

    println!("Verified block {}", results.evm.execution_header[0].number);
    println!("Verified balance {}", results.evm.account[0].balance);

    Ok(())
}
```

What happens here:

- `init_batch(...)` anchors the request to a Bankai block
- the batch builder collects the exact bankai proof requests you need
- `.execute()` fetches one optimized `ProofBundle`
- `verify_batch_proof(...)` checks the whole chain of trust and returns verified outputs

## Which Page Should You Read Next?

- Read [Proof Bundles](proof-bundles.md) if you want to understand how the bundle is composed.
- Read [Verify Crate Guide](verify.md) if you care most about the trust boundary.
- Read [Supported Surfaces](supported-surfaces.md) if you want to know what chains and methods are available today.
- Read [Basic Bundle Example](../example/basic-bundle/README.md) if you want a multi-chain walkthrough.
- Read [API Client Overview](api-client.md) if you need raw endpoint access instead of the batch builder.
