# risk-core

Shared fixed-point risk types, instruments, positions, market snapshots,
verdicts, and schema descriptors for Riskflow.

`risk-core` is the workspace vocabulary crate. It owns the identifiers,
fixed-point numeric wrappers, instrument reference types, trusted market-data
snapshot, schema descriptors, and verdict enums that must mean the same thing
in pretrade checks, portfolio analytics, benchmarks, validation fixtures, and
external adapters.

Primary documentation:

- [risk-core crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-core.md)
- [Reference-data example](https://github.com/gregorian-09/riskflow/blob/master/risk-core/examples/reference_data_flow.rs)
- [End-to-end code flow](https://github.com/gregorian-09/riskflow/blob/master/docs/end_to_end_code_flow.md)
- [Architecture](https://github.com/gregorian-09/riskflow/blob/master/docs/architecture.md)
- [Schema and migration policy](https://github.com/gregorian-09/riskflow/blob/master/docs/schemas.md)

## What It Contains

- fixed-point `Price`, `Qty`, `Notional`, and `Timestamp`,
- copyable `InstrumentId`,
- startup-only `SymbolRegistry`,
- `InstrumentCatalog` and instrument specs,
- `Position` variants for dynamic holdings,
- `MarketSnapshot` trust checks,
- `RiskVerdict` and indeterminate/reject reasons,
- schema version descriptors.

## Design Contract

`risk-core` does not run policy and does not perform I/O. It provides the
stable contract used by higher-level crates:

- adapters resolve venue symbols into `InstrumentId` before the hot path,
- checks use fixed-point wrappers for limit comparisons,
- market data is consumed through trust-checking accessors,
- unsupported or untrusted inputs become explicit indeterminate reasons,
- schema descriptors identify external records emitted or consumed by the
  workspace.

## Fixed-Point Arithmetic

```rust
use risk_core::{Notional, Price, Qty};

let notional = Notional::checked_linear(Price::new(100), Qty::new(5), 1)
    .expect("small fixture values do not overflow");

assert_eq!(notional.raw(), 500);
```

Overflow returns `None`; downstream crates convert that condition into a
fail-closed `RiskVerdict::Indeterminate`.

## Symbol And Instrument Setup

```rust
use risk_core::{
    CurrencyId, EquitySpec, Instrument, InstrumentCatalog, InstrumentId,
    SymbolKey, SymbolRegistry,
};

let symbol = SymbolKey {
    venue: "XNYS".to_owned(),
    symbol: "IBM".to_owned(),
};
let instrument = Instrument::Equity(EquitySpec {
    instrument_id: InstrumentId(1),
    settlement_currency: CurrencyId(840),
});

let mut registry = SymbolRegistry::new();
registry.register(symbol.clone(), InstrumentId(1)).unwrap();

let mut catalog = InstrumentCatalog::new();
catalog.insert(instrument).unwrap();

let resolved = registry.resolve(&symbol).unwrap();
assert_eq!(catalog.get(resolved), Some(instrument));
```

## Trusted Market Snapshot

```rust
use risk_core::{
    DataQuality, InstrumentId, MarketPrice, MarketSnapshot, Notional, Price,
    Timestamp,
};

let mut market = MarketSnapshot::new(10, 10, 10);
market.insert_price(
    InstrumentId(1),
    MarketPrice::clean(Price::new(100), Timestamp(5)),
);
market.set_aggregate_notional(
    Notional::new(0),
    Timestamp(5),
    DataQuality::clean(),
);

assert_eq!(
    market.trusted_price(InstrumentId(1), Timestamp(10)).unwrap(),
    Price::new(100)
);
```

Consumers should call trusted accessors instead of reading raw maps. Missing,
stale, low-quality, or source-disagreed inputs are represented as typed
indeterminate reasons.

## Public API Map

| Area | Main Types |
|---|---|
| Numeric wrappers | `Price`, `Qty`, `Notional`, `Timestamp` |
| Reference data | `InstrumentId`, `Instrument`, `InstrumentCatalog`, `SymbolRegistry` |
| Market data | `MarketSnapshot`, `MarketPrice`, `DataQuality` |
| Decisions | `RiskVerdict`, `RiskWeight`, `RejectReason`, `IndeterminateReason` |
| Schemas | `SchemaVersion`, `SchemaRecordKind`, `current_schema` |

## Read Next

- [Full crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-core.md) for module-by-module behavior.
- [End-to-end code flow](https://github.com/gregorian-09/riskflow/blob/master/docs/end_to_end_code_flow.md) for how `risk-core` feeds pretrade and analytics.
- [risk-pretrade README](https://github.com/gregorian-09/riskflow/blob/master/risk-pretrade/README.md) for order-evaluation usage.

## Verify

```bash
cargo test -p risk-core --all-features
cargo run -p risk-core --example reference_data_flow
RUSTDOCFLAGS="-D warnings" cargo doc -p risk-core --all-features --no-deps
```
