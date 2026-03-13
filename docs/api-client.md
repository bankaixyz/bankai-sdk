# API Client Overview

Use the batch builder by default.

Reach for `sdk.api.*` when you want direct access to the Bankai HTTP surface instead of an assembled `ProofBundle`.

## When The API Client Is The Right Tool

Use the raw API client when you want to:

- inspect API state before building a bundle
- discover active chains dynamically
- fetch snapshots, selectors, or proof payloads directly
- work with request DTOs yourself
- debug or explore the API surface

Use the batch builder when you want:

- the shortest path to verified application data
- bundle assembly
- one object that `bankai-verify` can consume directly

## Namespace Layout

`ApiClient` exposes:

- `blocks()`
- `chains()`
- `health()`
- `stats()`
- `ethereum()`
- `op_stack()`

## Recommended Inspection Flow

1. inspect chain support
2. inspect the latest Bankai block
3. inspect the finalized Ethereum or OP snapshot you care about
4. only then request a specific proof payload if needed

```rust
# async fn example(sdk: &bankai_sdk::Bankai) -> Result<(), Box<dyn std::error::Error>> {
use bankai_types::api::ethereum::BankaiBlockFilterDto;

let chains = sdk.api.chains().list().await?;
let latest_bankai_block = sdk.api.blocks().latest_number().await?;
let finalized = BankaiBlockFilterDto::finalized();

let execution_snapshot = sdk.api.ethereum().execution().snapshot(&finalized).await?;
let base_snapshot = sdk.api.op_stack().snapshot("base", &finalized).await?;

# let _ = (chains, latest_bankai_block, execution_snapshot, base_snapshot);
# Ok(())
# }
```

This tells you:

- which chains the API says are active
- what the latest Bankai anchor is
- what Ethereum and OP snapshots look like at a finalized Bankai view

## Common Namespace Uses

### `blocks()`

Use `blocks()` to:

- list or fetch Bankai blocks
- fetch full block payloads
- fetch Bankai block proofs and MMR proofs

### `chains()`

Use `chains()` to:

- discover active chain integrations
- avoid hardcoding chain support assumptions into your app

### `stats()`

Use `stats()` to:

- inspect high-level system coverage
- inspect block-level stats and root summaries

### `ethereum()`

Use `ethereum()` to:

- inspect root, beacon, and execution snapshots
- fetch sync-committee information
- request Ethereum MMR or light-client proof payloads directly

### `op_stack()`

Use `op_stack()` to:

- inspect OP snapshots
- fetch merkle proofs for committed OP clients
- fetch OP MMR proofs and light-client bundles directly

## Selectors And Filters

Low-level API flows use `BankaiBlockFilterDto`.

Useful starting points:

- `BankaiBlockFilterDto::finalized()`
- `BankaiBlockFilterDto::justified()`
- `BankaiBlockFilterDto::latest()`
- `BankaiBlockFilterDto::with_bankai_block_number(...)`

For most trust-sensitive inspection and proof requests, use `finalized`.

## Relationship To The Batch Builder

Think of the two layers like this:

- `sdk.api.*` lets you inspect and request raw Bankai data
- the batch builder wraps those raw surfaces plus your configured RPCs into one verifier-ready bundle

If you are building an application feature, the batch builder is usually the better default.
