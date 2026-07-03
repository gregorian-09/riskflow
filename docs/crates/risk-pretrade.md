# `risk-pretrade`

`risk-pretrade` is the synchronous order-entry risk gate.

## Responsibilities

- Store immutable limit snapshots behind `ArcSwap`.
- Evaluate fixed-order risk checks.
- Fail closed on untrusted inputs.
- Emit audit records.
- Provide observability counters and structured event payloads.
- Parse static/file-backed v1 limit tables.

## Evaluation Inputs

`EvaluateRequest` contains:

- static `Instrument`,
- order `Qty`,
- current position,
- available margin,
- submitted order price,
- `MarketSnapshot`,
- current timestamp.

## Check Pipeline

1. Trading enabled.
2. Instrument risk weight using trusted market data.
3. Per-order notional.
4. Aggregate notional snapshot.
5. Position limit.
6. Margin.
7. Fat-finger price band.

Any check may return `Reject` or `Indeterminate`.

## Limit File Format

```text
schema_version,1,0,0
aggregate_notional,10000
per_order_notional,1,1000
max_abs_position,1,50
fat_finger_band_bps,1,250
initial_margin_per_unit,1,10
```

## Observability

Use `PretradeGate::metrics_snapshot()` and structured event types from
`risk_pretrade::observability`. The crate keeps logging and metrics backends
out of the hot path.

## Verification

```bash
cargo test -p risk-pretrade --all-features
cargo test -p risk-pretrade --test golden_pretrade
cargo test -p risk-pretrade --test adapter_contracts
cargo test -p risk-pretrade --test adversarial_pretrade
cargo run -p risk-pretrade --example end_to_end_adapter
```
