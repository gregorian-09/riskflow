# `risk-portfolio`

`risk-portfolio` is the offline analytics crate.

## Responsibilities

- Historical `VaR`.
- Parametric normal `VaR`.
- Seeded Monte Carlo `VaR`.
- Marginal and component `VaR`.
- Deterministic stress scenarios.
- Performance summaries, ratios, and drawdowns.
- Cross-currency netting helpers.
- Optional Python bindings.

## Error Model

Convenience functions return `Option` for compact usage. Typed `try_*`
variants return `VarError` or `ScenarioError` where diagnostics matter.

## Determinism

Monte Carlo analytics require a `SimulationSeed`. Same input and seed produce
the same output.

## Example

```rust
use nalgebra::dmatrix;
use risk_portfolio::var::try_parametric_var_attribution;

let weights = [0.6, 0.4];
let covariance = dmatrix![0.04, 0.01; 0.01, 0.09];
let report = try_parametric_var_attribution(&weights, &covariance, 0.95)
    .expect("fixture covariance is valid");

let component_sum = report.component_var.iter().sum::<f64>();
assert!((report.portfolio_var - component_sum).abs() < 1e-12);
```

## Verification

```bash
cargo test -p risk-portfolio --all-features
cargo test -p risk-portfolio --test golden_stress
cargo test -p risk-portfolio --test golden_var
```
