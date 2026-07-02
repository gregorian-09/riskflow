# Hardening Checks

Run these checks before release:

```bash
cargo audit --db target/advisory-db
cargo deny check
cargo tarpaulin --workspace --out Lcov --output-dir coverage
cargo package --workspace --allow-dirty
```

## Advisory Policy

`cargo deny` fails on advisories, rejected licenses, unknown registries, and
unknown Git sources. The only advisory exception is `RUSTSEC-2024-0436` for
`paste`, which is pulled transitively by `nalgebra 0.33.x`. That `nalgebra`
line is used to preserve the workspace Rust 1.85 MSRV. Revisit the exception
when `nalgebra` can be upgraded under the workspace MSRV.

## Coverage

The current tarpaulin command excludes optional Python bindings because PyO3's
instrumented test link can require Python linker details that are not needed by
normal `cargo test --workspace --all-features`. The last local hardening run
reported 68.45% line coverage on the core Rust path.

## Semver

`cargo semver-checks --workspace` is not actionable until the crates have a
published baseline. The current result is:

```text
risk-core not found in registry (crates.io)
```

Run semver checks after the first publish, or provide an explicit baseline rev
when comparing against a previous local release commit.
