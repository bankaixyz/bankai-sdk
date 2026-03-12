# Ethereum Light Clients

Bankai mirrors Ethereum finality through three selectors: `latest`, `justified`, and `finalized`.

That is the right mental model for reading Bankai Ethereum data.

## Sync Committees

Ethereum updates are advanced with sync committee signatures.

After each epoch, Bankai verifies the sync committee signature, backfills the new headers into the MMR, and checks that the chain stays linked correctly.

## Finality Modes

Bankai exposes the same three views you already use on Ethereum:

| Mode | Meaning |
| --- | --- |
| `latest` | The newest Bankai view available |
| `justified` | A stronger consensus checkpoint, but not the strongest one |
| `finalized` | The strongest Bankai view derived from Ethereum finality |

The important ordering is:

```text
latest >= justified >= finalized
```

## What Finalized Means

Use `finalized` by default for trust-sensitive verification.

Bankai follows the native protocol here. `latest`, `justified`, and `finalized` mean the same thing they mean on Ethereum.

## How This Connects To Bankai

In Bankai:

- beacon data reflects Ethereum consensus state
- execution data reflects Ethereum execution state
- both are committed into the verified Bankai block

So when you select a finalized Bankai view, you are anchoring the proof flow to Ethereum finality instead of to a weaker, newer view.

## Practical Guidance

- start with `finalized`
- use `justified` or `latest` only when your application explicitly wants fresher but less settled data
- keep in mind that selectors belong to the Bankai view used to anchor the proof bundle

If you want to inspect selectors directly, the low-level API exposes them through `BankaiBlockFilterDto` and the related root, snapshot, and proof endpoints.
