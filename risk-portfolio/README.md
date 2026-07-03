# risk-portfolio

Offline portfolio analytics for Riskflow.

Read the full guide:

- [risk-portfolio crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-portfolio.md)
- [Model Validation](https://github.com/gregorian-09/riskflow/blob/master/docs/model_validation.md)
- [Validation](https://github.com/gregorian-09/riskflow/blob/master/docs/validation.md)

## Quick Example

```rust
use risk_portfolio::var::{SimulationSeed, monte_carlo_var};

let var = monte_carlo_var(0.0, 0.02, 0.95, 1_000, SimulationSeed(42));
assert!(var.is_some());
```

## Verify

```bash
cargo test -p risk-portfolio --all-features
```
