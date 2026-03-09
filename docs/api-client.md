# API Client Overview

Use the batch builder by default.

Use `sdk.api.*` when you need one of these:

- raw endpoint access
- custom request DTOs
- direct proof payload retrieval
- API inspection without building a `ProofBundle`

## Namespace Layout

`ApiClient` exposes these namespaces:

- `blocks()`
- `chains()`
- `health()`
- `stats()`
- `ethereum()`
- `op_stack()`

## Typical Uses

### Blocks

Use `blocks()` to discover Bankai blocks and fetch block proofs.

```rust
# async fn example(sdk: &bankai_sdk::Bankai) -> Result<(), Box<dyn std::error::Error>> {
let latest = sdk.api.blocks().latest_number().await?;
let full_block = sdk.api.blocks().full(latest).await?;
let proof = sdk.api.blocks().proof(latest).await?;
# let _ = (full_block, proof);
# Ok(())
# }
```

### Ethereum

Use `ethereum()` when you want direct light-client proof or MMR proof requests.

```rust
use bankai_sdk::HashingFunction;
use bankai_types::api::ethereum::{BankaiBlockFilterDto, EthereumLightClientProofRequestDto};
use bankai_types::common::ProofFormat;

# async fn example(sdk: &bankai_sdk::Bankai) -> Result<(), Box<dyn std::error::Error>> {
let filter = BankaiBlockFilterDto::with_bankai_block_number(123);
let request = EthereumLightClientProofRequestDto {
    filter,
    hashing_function: HashingFunction::Keccak,
    header_hashes: vec!["0x...".to_string()],
    proof_format: ProofFormat::Bin,
};

let proof = sdk
    .api
    .ethereum()
    .execution()
    .light_client_proof(&request)
    .await?;
# let _ = proof;
# Ok(())
# }
```

### OP Stack

Use `op_stack()` for OP-specific endpoints like snapshots, merkle proofs, and light-client bundles.

```rust
use bankai_types::api::ethereum::BankaiBlockFilterDto;

# async fn example(sdk: &bankai_sdk::Bankai) -> Result<(), Box<dyn std::error::Error>> {
let filter = BankaiBlockFilterDto::with_bankai_block_number(123);
let snapshot = sdk.api.op_stack().snapshot("base", &filter).await?;
# let _ = snapshot;
# Ok(())
# }
```

## Relationship to the Batch Builder

The batch builder wraps the API client plus your configured RPC providers and returns a `ProofBundle` that `bankai-verify` can consume directly.

Reach for the API client when you want control over the raw request/response layer. Reach for the batch builder when you want the simplest verified application flow.
