# risk-pretrade

Synchronous pretrade risk gate for Riskflow.

`risk-pretrade` is the order-entry crate. It evaluates a concrete sequence of
checks against trusted market data and immutable limit snapshots. It emits
audit records and observability payloads that adapters can export.

Read the full guide:

- [risk-pretrade crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-pretrade.md)
- [Validation](https://github.com/gregorian-09/riskflow/blob/master/docs/validation.md)
- [Observability](https://github.com/gregorian-09/riskflow/blob/master/docs/observability.md)

## Check Pipeline

1. Trading enabled.
2. Instrument risk weight.
3. Per-order notional.
4. Aggregate notional snapshot.
5. Position limit.
6. Margin.
7. Fat-finger price band.

Any missing, stale, low-quality, overflowing, or unsupported input fails
closed.

## Quick Example

Run the end-to-end adapter example:

```bash
cargo run -p risk-pretrade --example end_to_end_adapter
```

## Read Next

Use the full crate guide for:

- `EvaluateRequest` construction,
- schema-aware limit files,
- observability event construction,
- adapter tests,
- adding checks and limit records.

## Verify

```bash
cargo test -p risk-pretrade --all-features
```
