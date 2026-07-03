# risk-core

Shared fixed-point risk types, instruments, positions, market snapshots,
verdicts, and schema descriptors for Riskflow.

Read the full guide:

- [risk-core crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-core.md)
- [Architecture](https://github.com/gregorian-09/riskflow/blob/master/docs/architecture.md)

## Quick Example

```rust
use risk_core::{Notional, Price, Qty};

let notional = Notional::checked_linear(Price::new(100), Qty::new(5), 1)
    .expect("small fixture values do not overflow");

assert_eq!(notional.raw(), 500);
```

## Verify

```bash
cargo test -p risk-core --all-features
```
