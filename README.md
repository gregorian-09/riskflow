# Risk Workspace

Risk is a Rust workspace for multi-asset risk checks and offline portfolio
analytics. It is designed as a companion to Orderflow: Orderflow describes
market state, while Risk decides whether an order can be sent and how much
exposure a book carries.

## Crates

- `risk-core`: fixed-point identifiers, instruments, positions, market
  snapshots, data-quality handling, and verdict types.
- `risk-pretrade`: synchronous pretrade gate with an `ArcSwap` limit table and
  fail-closed checks for notional, aggregate exposure, position, margin, and
  fat-finger limits.
- `risk-portfolio`: batch analytics for performance, historical/parametric/
  seeded Monte Carlo VaR, VaR attribution, deterministic stress scenarios, and
  cross-currency netting helpers.
- `risk-bench`: Criterion harnesses for pretrade gate latency measurement.

## Scope

The v1 critical path covers equities, spot FX, spot crypto, futures, and
perpetual swaps. Options are represented in the taxonomy but intentionally
return an indeterminate risk weight until a separate `risk-options` crate
exists behind the shared exposure boundary.

Risk does not implement regulatory capital, credit risk, liquidity risk, live
exchange margin-schedule ingestion, or AI/ML risk-determining models in v1.
Missing, stale, low-quality, or unsupported inputs fail closed through
`RiskVerdict::Indeterminate`.

## Quality Gates

The expected local verification set is:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --examples --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench -p risk-bench --bench evaluate -- --test
cargo run -p risk-bench --release -- --iterations 50000
```

Benchmark reporting methodology lives in `docs/benchmarks.md`; validation
fixtures live in `docs/validation.md`; hardening checks live in
`docs/hardening.md`; named constants and test fixture values are documented in
`docs/constants.md`.

The CI workflow also checks that `risk-pretrade` and `risk-portfolio` do not
depend on `risk-options`.
