# World ID Root (Sepolia) — Bankai SDK Example

This example retrieves the **World ID root** at a **specific Sepolia execution block** and verifies it using Bankai proofs.

## What this demonstrates (end-to-end)

- **Stateless light client architecture**: Bankai proofs are stateless light client proofs.
- **Access by verification**: because of this construction, you can access on-chain data by verifying a Bankai proof.
- **No destination-chain state**: crucially, these light client proofs are valid without relying on a smart contract to store state.
- **Decommit arbitrary data via the SDK**: verify the proof, then decommit any data you want (here: the World ID root at a specific Sepolia block).
- **Prove in a zkVM**: this verification + decommit can be proven inside a zkVM.
- **Bring the zk proof anywhere**: the resulting zkVM proof can be verified on any chain (Solana, Sui, Avalanche, L3s/appchains) or in client-side circuits.
- **No destination infra required**: because stateless light clients don’t require on-chain infrastructure, you can move the World ID root without deploying/maintaining a light client on the destination chain.
- **Trustless root anywhere**: verify the zkVM proof anywhere and immediately have trustless access to the root.

## Run it

From the repo root:

```bash
export EXECUTION_RPC="https://sepolia.infura.io/v3/YOUR_KEY"
cargo run -p bankai-example-worldid-root
```

The program prints the **verified** World ID root value for the configured block.

