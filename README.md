# Bankai SDK

Bankai SDK lets you fetch proof bundles for on-chain data and verify
them with `bankai-verify`. This README covers the shortest path to
using the SDK: install the crates, apply the required
`ethereum_hashing` patch, configure the client, and run a few first
proof flows. Full long-form guides now live in the
[Bankai docs](https://docs.bankai.xyz/docs/sdk).

## Install

Start by pinning the current release tag from this repository. You
also need the patched `ethereum_hashing` crate.

```toml
[dependencies]
bankai-sdk = { git = "https://github.com/bankaixyz/bankai-sdk", tag = "v0.1.2.1" }
bankai-verify = { git = "https://github.com/bankaixyz/bankai-sdk", tag = "v0.1.2.1" }
bankai-types = { git = "https://github.com/bankaixyz/bankai-sdk", tag = "v0.1.2.1" }
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e", default-features = false, features = ["portable"] }

[patch.crates-io]
ethereum_hashing = { git = "https://github.com/bankaixyz/ethereum_hashing", rev = "c457c3e927cc146d7bc91e944cf6d9c55b05d45e" }
```

Add `bankai-types` only if you want the shared input, result, or API
types directly in your own code. Most applications start with
`bankai-sdk` and `bankai-verify`.

## Configure the SDK

Create one `Bankai` instance, then pass only the RPCs needed for the
proofs you plan to request.

```rust
use std::collections::BTreeMap;

use bankai_sdk::{Bankai, Network};

let mut op_rpcs = BTreeMap::new();
op_rpcs.insert("base".to_string(), "https://sepolia.base.org".to_string());

let bankai = Bankai::new(
    Network::Sepolia,
    Some("https://sepolia.infura.io/v3/YOUR_KEY".to_string()),
    Some("https://sepolia.beacon-api.example.com".to_string()),
    Some(op_rpcs),
);
```

## Verify your first proof

This is the shortest useful end-to-end flow: fetch a proof bundle for
an execution header and account, then verify it locally.

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
        .init_batch(None, HashingFunction::Keccak)
        .await?
        .ethereum_execution_header(9_231_247)
        .ethereum_account(9_231_247, Address::ZERO)
        .execute()
        .await?;

    let results = verify_batch_proof(proof_bundle)?;

    println!("Verified block {}", results.evm.execution_header[0].number);
    println!(
        "Verified balance {} at block {}",
        results.evm.account[0].account.balance,
        results.evm.account[0].block.block_number
    );

    Ok(())
}
```

Use `HashingFunction::Keccak` as the default starting point. Switch to
`Poseidon` when you are targeting Cairo-native verification.

## Inspect the raw API

Reach for `bankai.api` when you want to inspect chain support, query
snapshots, or debug the Bankai surface before you build a bundle.

```rust
use bankai_sdk::{Bankai, Network};
use bankai_types::api::ethereum::BankaiBlockFilterDto;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let bankai = Bankai::new(Network::Sepolia, None, None, None);
let finalized = BankaiBlockFilterDto::finalized();

let chains = bankai.api.chains().list().await?;
let latest_bankai = bankai.api.blocks().latest_number().await?;
let execution = bankai.api.ethereum().execution().snapshot(&finalized).await?;
let base = bankai.api.op_stack().snapshot("base", &finalized).await?;

println!("Chains: {}", chains.len());
println!("Latest Bankai block: {}", latest_bankai);
println!("Execution height: {}", execution.end_height);
println!("Base height: {}", base.end_height);
# Ok(())
# }
```

For production data retrieval, prefer the batch builder. It assembles
one verifier-ready `ProofBundle` instead of making you stitch raw proof
payloads together yourself.

## Read next

The canonical guides now live in `bankai-docs`.

- [SDK quickstart](https://docs.bankai.xyz/docs/sdk/quickstart)
- [Proof bundles](https://docs.bankai.xyz/docs/sdk/proof-bundles)
- [Verify a proof](https://docs.bankai.xyz/docs/sdk/verify-a-proof)
- [API client](https://docs.bankai.xyz/docs/sdk/api-client)
- [Supported chains](https://docs.bankai.xyz/docs/sdk/supported-surfaces)
- [Stateless light clients](https://docs.bankai.xyz/docs/concepts/stateless-light-clients)
- [Bankai blocks](https://docs.bankai.xyz/docs/concepts/bankai-blocks)
