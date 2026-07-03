# Changelog

All notable changes to Riskflow are tracked here. The project follows
Conventional Commits for commit messages and keeps release notes focused on
behavior, validation, and operational impact.

## Unreleased

### Added

- Workspace split into `risk-core`, `risk-pretrade`, `risk-portfolio`, and
  `risk-bench`.
- Fixed-point core types for price, quantity, notional, timestamps, and
  copyable instrument ids.
- Instrument and position taxonomies for equities, spot FX, spot crypto,
  futures, perpetual swaps, and v1 option placeholders.
- Market snapshot trust checks for staleness, source agreement, FX conversion,
  aggregate exposure snapshots, and upstream data-quality flags.
- Pretrade gate with per-order notional, aggregate notional, position limit,
  margin, and fat-finger checks.
- Audit records for order decisions, limit updates, and trading state changes.
- Pretrade observability counters, trace context, structured events, and alert
  severity mapping.
- File-backed and static limit sources with schema version handling.
- Portfolio performance analytics, historical `VaR`, parametric `VaR`, seeded
  Monte Carlo `VaR`, component `VaR`, deterministic stress scenarios, and
  cross-currency netting helpers.
- Golden validation fixtures for pretrade decisions, stress scenarios, and
  historical `VaR`.
- Adversarial pretrade tests for overflow, degraded feeds, stale aggregates,
  and disabled trading.
- Benchmark harness and release evidence scripts.
- CI workflows for Rust quality, governance checks, release evidence, and
  self-hosted benchmark matrix runs.
- Public documentation set with architecture, getting started, crate guides,
  operations, observability, schema policy, validation, hardening, governance,
  security review, and contributor guidance.

### Not Included

- Options pricing and Greeks beyond unsupported v1 taxonomy.
- C, Python, or Java-style ABI surfaces outside the Rust crate APIs.
- Regulatory capital, credit risk, liquidity risk, live margin ingestion, and
  AI/ML risk-determining models.
