# Model Validation Methodology

This document is the model-validation pack for the v1 risk workspace. It is
written so an independent reviewer can reproduce the implemented analytics,
challenge assumptions, and record sign-off without relying on source-code
comments.

## Scope

Validated v1 models:

- fixed-point linear notional exposure,
- fail-closed pretrade limit checks,
- aggregate notional snapshot checks,
- futures and perpetual initial-margin checks,
- fat-finger price-band checks,
- historical `VaR`,
- parametric normal `VaR`,
- seeded Monte Carlo `VaR`,
- marginal and component parametric `VaR`,
- deterministic stress scenarios,
- performance ratios and drawdown analytics.

Explicitly excluded:

- options pricing and Greeks,
- regulatory capital,
- credit risk,
- liquidity risk,
- AI/ML risk-determining models.

## Methodology Inventory

| Method | Assumption | Independent Check |
|---|---|---|
| Linear notional | Exposure is `abs(price * qty * multiplier)` using checked fixed-point arithmetic. | `risk-core/tests/numeric_properties.rs` and adversarial overflow tests. |
| Market trust | Missing, stale, or low-quality market inputs fail closed. | `risk-core::market` tests and `risk-pretrade/tests/adversarial_pretrade.rs`. |
| Aggregate notional | Whole-book cross-currency exposure is supplied as a bounded-staleness snapshot. | Golden pretrade fixtures and aggregate stale tests. |
| Historical `VaR` | Tail loss is selected deterministically from sorted periodic returns. | `risk-portfolio/tests/golden_var.rs`. |
| Parametric `VaR` | Return distribution is approximated as normal with named confidence-level z-scores. | Unit tests for supported confidence levels and attribution sum checks. |
| Monte Carlo `VaR` | Simulation is deterministic for a fixed seed. | Seed reproducibility tests. |
| Stress scenarios | Return shocks are additive and applied to an ordered asset universe. | `risk-portfolio/tests/golden_stress.rs`. |

## Validation Commands

```bash
cargo test --workspace --all-features
cargo test --workspace --examples --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

## Reviewer Challenge Checklist

- Recompute fixed-point notional examples by hand from fixture values.
- Confirm `i64::MIN` and multiplication overflow fail closed.
- Confirm unsupported options are indeterminate end-to-end.
- Recompute historical `VaR` from `historical_var.csv`.
- Confirm component `VaR` sums to portfolio `VaR` within tolerance.
- Confirm stale price, stale aggregate, and bad quality inputs never pass.
- Confirm benchmark numbers are produced by the documented command and machine.

## Sign-Off Record

| Role | Name | Organization | Date | Decision | Notes |
|---|---|---|---|---|---|
| Model owner | Pending | Pending | Pending | Pending | Implementation pack prepared. |
| Independent validator | Pending | Pending | Pending | Pending | Must be completed by a reviewer outside the implementation author. |
| Risk approver | Pending | Pending | Pending | Pending | Required before production use. |

Do not mark this pack approved until the independent validator has reproduced
the validation commands, reviewed the assumptions above, and recorded a
decision in the table.
