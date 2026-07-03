# `risk-core`

`risk-core` is the shared type and trust-boundary crate.

## Responsibilities

- Fixed-point primitives: `Price`, `Qty`, `Notional`, `Timestamp`.
- Stable identity: `InstrumentId`.
- Startup symbol mapping: `SymbolRegistry`.
- Instrument taxonomy: `Instrument` and asset-specific spec structs.
- Dynamic holdings: `Position`.
- Market trust: `MarketSnapshot`, `MarketPrice`, `DataQuality`.
- Results: `RiskWeight`, `RiskVerdict`, `RejectReason`,
  `IndeterminateReason`.
- External schema versions.

## What It Does Not Own

- Order-entry policy,
- limit table storage,
- portfolio analytics,
- option pricing,
- FFI surfaces.

## Important Types

| Type | Purpose |
|---|---|
| `InstrumentId` | Copyable id used inside hot paths. |
| `SymbolRegistry` | Startup-only mapping from external symbol keys to `InstrumentId`. |
| `Instrument` | Static reference data and risk-weight behavior. |
| `Position` | Dynamic position state. |
| `MarketSnapshot` | Trusted prices, FX rates, quality flags, aggregate notional. |
| `RiskVerdict` | Final pretrade decision. |
| `SchemaVersion` | Version identifier for external records. |

## Failure Behavior

`risk-core` never treats uncertainty as zero risk. Missing prices, stale data,
bad data quality, unsupported options, unknown symbols, and arithmetic overflow
produce explicit indeterminate reasons.

## Example

```rust
use risk_core::{InstrumentId, Notional, Price, Qty};

let notional = Notional::checked_linear(Price::new(100), Qty::new(-5), 1)
    .expect("small fixture values do not overflow");

assert_eq!(InstrumentId(7).raw(), 7);
assert_eq!(notional.raw(), 500);
```

## Verification

```bash
cargo test -p risk-core --all-features
cargo test -p risk-core --test numeric_properties
```
