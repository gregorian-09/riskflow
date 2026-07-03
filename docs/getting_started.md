# Getting Started

This guide walks through the fastest path to understanding and running
Riskflow.

## 1. Understand The Split

Riskflow has two primary runtime surfaces:

- `risk-pretrade`: synchronous, fixed-point, fail-closed checks for order entry.
- `risk-portfolio`: offline analytics for reports, validation, and backtests.

Both depend on `risk-core`, which owns shared types and trust boundaries.

## 2. Run The Tests

```bash
cargo test --workspace --all-features
```

This runs:

- core unit tests,
- numeric property tests,
- pretrade gate tests,
- adapter contract tests,
- adversarial fail-closed tests,
- portfolio analytics tests,
- golden validation fixtures.

## 3. Run The End-To-End Adapter Example

```bash
cargo run -p risk-pretrade --example end_to_end_adapter
```

The example shows the normal integration shape:

1. Receive venue and symbol fields from an order-entry adapter.
2. Resolve them through `SymbolRegistry`.
3. Fetch static instrument reference data from `InstrumentCatalog`.
4. Build an `EvaluateRequest`.
5. Evaluate through `PretradeGate`.
6. Emit audit and observability records.

## 4. Inspect The Validation Fixtures

```bash
cargo test -p risk-pretrade --test golden_pretrade
cargo test -p risk-pretrade --test adapter_contracts
cargo test -p risk-pretrade --test adversarial_pretrade
cargo test -p risk-portfolio --test golden_stress
cargo test -p risk-portfolio --test golden_var
```

The fixtures are deliberately small enough to audit by hand.

## 5. Read The Architecture

Read [Architecture](architecture.md) after the first test run. It explains:

- crate boundaries,
- data flow,
- fail-closed behavior,
- fixed-point arithmetic,
- schema policy,
- deferred crates.

## 6. Generate Release Evidence

```bash
scripts/release_evidence.sh target/release-evidence
```

This creates logs for tests, examples, Clippy, rustdoc, audit, deny, package
verification, benchmark smoke, and governance checks.
