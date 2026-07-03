# risk-portfolio

Offline portfolio analytics for Riskflow.

`risk-portfolio` is the reporting and analytics crate. It uses `f64` and matrix
math because it is not part of the fixed-point pretrade hot path. Simulation
analytics are still deterministic through explicit seeds.

Read the full guide:

- [risk-portfolio crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-portfolio.md)
- [Model Validation](https://github.com/gregorian-09/riskflow/blob/master/docs/model_validation.md)
- [Validation](https://github.com/gregorian-09/riskflow/blob/master/docs/validation.md)

## Analytics Surface

- performance summaries,
- historical `VaR`,
- parametric `VaR`,
- seeded Monte Carlo `VaR`,
- component and marginal `VaR`,
- covariance matrices,
- deterministic stress scenarios,
- cross-currency netting helpers.

## Quick Example

```rust
use risk_portfolio::var::{SimulationSeed, monte_carlo_var};

let var = monte_carlo_var(0.0, 0.02, 0.95, 1_000, SimulationSeed(42));
assert!(var.is_some());
```

## Read Next

Use the full crate guide for:

- typed error variants,
- attribution examples,
- stress scenario examples,
- covariance flow,
- Python binding extension rules.

## Verify

```bash
cargo test -p risk-portfolio --all-features
```
