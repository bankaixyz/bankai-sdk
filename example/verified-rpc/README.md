# Verified RPC Example

This example crate wraps execution-layer JSON-RPC header retrieval and verifies
historical headers using Bankai MMR proofs. It focuses on a minimal, transport-
isolated client that can be reused in WASM-friendly contexts.

## Features

- Fetches execution headers via JSON-RPC and computes the canonical header hash.
- Fetches Bankai STWO block proofs and MMR proofs to verify header inclusion.
- Returns a `VerifiedHeader` with proof metadata.
- Provides a `call` passthrough for unverified JSON-RPC calls.

## Running the Demo (Native)

```bash
cargo run -p bankai-example-verified-rpc --features native
```

### Required Environment Variables

- `RPC_URL`: Execution JSON-RPC endpoint.
- `BLOCK_NUMBER`: Historical block number to verify.

### Optional Environment Variables

- `BANKAI_BLOCK_NUMBER`: Bankai block height to anchor proofs to (defaults to latest).
- `BANKAI_API_BASE`: Override the default Bankai API base URL.

Example:

```bash
RPC_URL="https://sepolia.infura.io/v3/YOUR_KEY" \
BLOCK_NUMBER=5200000 \
cargo run -p bankai-example-verified-rpc --features native
```

## Library Usage

```no_run
use bankai_example_verified_rpc::VerifiedRpcClient;
use bankai_sdk::Network;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = VerifiedRpcClient::new(Network::Sepolia, "https://rpc".to_string(), None);
let verified = client.get_block_by_number_verified(5_200_000, None).await?;
println!("Verified header hash: {:?}", verified.header_hash);
# Ok(())
# }
```

## WASM Notes

The core verification flow is transport-agnostic. Build with
`--no-default-features --features wasm` and supply a custom JSON-RPC transport
as needed.
