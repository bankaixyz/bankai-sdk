# World ID Root Example

This example shows a focused OP Stack flow:

1. connect to a local Bankai API
2. fetch an OP Stack account proof for Base
3. verify the proof bundle
4. print the verified header and account balance

## Requirements

- a Bankai API available at `http://localhost:8080`
- `BASE_RPC` set to a Base RPC endpoint

## Run

```bash
export BASE_RPC="https://mainnet.base.org"
cargo run -p bankai-example-worldid-root
```

The example uses `Network::Local` on purpose so it is aligned with a local Bankai deployment.

If you only want a local Bankai API endpoint but still need Sepolia semantics for Ethereum-side requests, use `Bankai::new_with_base_url(Network::Sepolia, "http://localhost:8080".to_string(), ...)` instead.
