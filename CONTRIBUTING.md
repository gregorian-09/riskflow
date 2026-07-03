# Contributing to Riskflow

Riskflow is a risk library, so contributions are judged by correctness,
auditability, deterministic behavior, and operational clarity before cleverness
or feature breadth.

## Development Setup

Required:

- Rust toolchain compatible with the workspace `rust-version`,
- `cargo fmt`,
- `cargo clippy`,
- `cargo audit`,
- `cargo deny`.

Recommended:

- GitHub CLI for workflow inspection,
- a local target directory with enough space for docs and package checks.

## Contribution Workflow

1. Read [Architecture](docs/architecture.md).
2. Identify which crate owns the behavior.
3. Keep changes scoped to that crate and its tests.
4. Add or update validation fixtures for externally visible behavior.
5. Update the relevant crate README and docs page.
6. Run the quality gates listed in the root README.
7. Open a PR using the template.

## Design Rules

- Pretrade checks fail closed.
- Pretrade arithmetic is fixed-point and checked.
- Portfolio analytics may use `f64`, but simulation must be deterministic.
- No `unsafe` code.
- No trait objects or heap allocation in pretrade check logic.
- `risk-pretrade` and `risk-portfolio` must not depend on `risk-options`.
- Public APIs need rustdoc.
- Fallible public APIs should use typed errors when callers need diagnostics.

## Testing Expectations

Use the smallest test that proves the behavior:

- unit tests for local pure functions,
- integration tests for public workflows,
- golden fixtures for reviewable risk decisions,
- property tests for numeric boundary behavior,
- adversarial tests for fail-closed paths.

New market-data, parser, schema, or limit behavior should include malformed or
stale input tests.

## Documentation Expectations

Every user-facing change should update at least one of:

- root README,
- crate README,
- `docs/crates/*.md`,
- validation docs,
- operations or observability docs,
- changelog.

Documentation should explain:

- what the feature does,
- what it deliberately does not do,
- which crate owns it,
- how to verify it,
- how it fails closed.

## Pull Request Checklist

Before requesting review:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --examples --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
scripts/check_governance.sh
```

Run `cargo audit` and `cargo deny check` when dependency or release-facing
changes are included.

## Review Standard

Reviewers should look first for:

- fail-open behavior,
- unchecked fixed-point arithmetic,
- stale or missing market data being treated as trusted,
- accidental dependency boundary violations,
- missing validation fixtures,
- undocumented schema changes,
- benchmark claims without evidence.
