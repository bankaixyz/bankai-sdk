# World ID Root Example

This example is the current bridge between today's code and the bigger World ID story.

It does not yet implement the full "replicate World ID roots through a zkVM" flow. Instead, it shows the kind of verified multichain data access that flow will depend on:

- Base OP Stack proofs
- Ethereum execution proofs
- one consistent `verify_batch_proof(...)` trust boundary

## What The Example Does Today

The example binary in [`src/main.rs`](src/main.rs):

1. connects to a local Bankai API
2. fetches Base OP Stack proofs for:
   - an account
   - a storage slot
   - a transaction
   - a receipt
3. verifies the Base proof bundle
4. fetches Sepolia Ethereum execution proofs for:
   - an execution header
   - an account
   - a storage slot
5. verifies the Ethereum proof bundle
6. prints the verified outputs

It already demonstrates the cross-surface verification path the later World ID flow will use.

## Why This Example Matters

World ID is a good motivating story because it makes the payoff obvious:

- the source data lives on an OP Stack chain
- you want to verify it somewhere else
- you need a trust-minimized way to carry that data across environments

This example is the current first step in that direction.

## Run

Requirements:

- a Bankai API available at `http://localhost:8080`
- `BASE_RPC` set to a Base RPC endpoint
- `EXECUTION_RPC` set to a Sepolia execution RPC endpoint

```bash
export BASE_RPC="https://mainnet.base.org"
export EXECUTION_RPC="https://sepolia.infura.io/v3/YOUR_KEY"
cargo run -p bankai-example-worldid-root
```

## What To Notice In The Output

The output shows that the verifier can recover trusted values across both:

- Base OP Stack data
- Sepolia Ethereum execution data

That is exactly the capability the later World ID replication flow will build on.

## Why It Uses `Network::Local`

The current example is intentionally aligned with a local Bankai deployment.

If you only want a local Bankai API endpoint but still want Sepolia semantics on the Ethereum side, use `Bankai::new_with_base_url(Network::Sepolia, "http://localhost:8080".to_string(), ...)` instead.

## Read Next

- [Basic Bundle Example](../basic-bundle/README.md)
- [OP Stack Concepts](../../docs/concepts-op-stack.md)
- [World ID Replicator Placeholder](../worldid-replicator/README.md)
