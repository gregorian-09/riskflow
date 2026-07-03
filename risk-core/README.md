# risk-core

Shared fixed-point risk types, instruments, positions, market snapshots,
verdicts, and schema descriptors for Riskflow.

`risk-core` is the vocabulary crate. It owns the types that must remain stable
across pretrade checks, portfolio analytics, validation fixtures, adapters, and
future bindings.

Read the full guide:

- [risk-core crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-core.md)
- [Architecture](https://github.com/gregorian-09/riskflow/blob/master/docs/architecture.md)

## What It Contains

- fixed-point `Price`, `Qty`, `Notional`, and `Timestamp`,
- copyable `InstrumentId`,
- startup-only `SymbolRegistry`,
- `InstrumentCatalog` and instrument specs,
- `Position` variants for dynamic holdings,
- `MarketSnapshot` trust checks,
- `RiskVerdict` and indeterminate/reject reasons,
- schema version descriptors.

## Quick Example

```rust
use risk_core::{Notional, Price, Qty};

let notional = Notional::checked_linear(Price::new(100), Qty::new(5), 1)
    .expect("small fixture values do not overflow");

assert_eq!(notional.raw(), 500);
```

## Read Next

Use the full crate guide for:

- symbol resolution lifecycle,
- market snapshot trust behavior,
- risk weight flow,
- schema extension rules,
- tests to read first.

## Verify

```bash
cargo test -p risk-core --all-features
```
