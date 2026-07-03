# Hardening Policy

Run these checks before release:

```bash
cargo audit --db target/advisory-db
cargo deny check
cargo tarpaulin --workspace --out Lcov --output-dir coverage
cargo package -p risk-core --allow-dirty
```

## Advisory Policy

`cargo deny` fails on advisories, rejected licenses, unknown registries, and
unknown Git sources. The only advisory exception is `RUSTSEC-2024-0436` for
`paste`, which is pulled transitively by `nalgebra 0.33.x`. That `nalgebra`
line is used to preserve the workspace Rust 1.85 MSRV. Revisit the exception
when `nalgebra` can be upgraded under the workspace MSRV.

## Coverage

Coverage reports are release evidence, not API documentation. Generate them
with `cargo tarpaulin` and attach the LCOV artifact to the release packet.
Optional Python bindings may require environment-specific linker setup; when
they are excluded from a coverage run, record that exclusion in the release
evidence rather than weakening the normal `cargo test --workspace --all-features`
gate.

## Semver

Run semver checks for every public release after a stable baseline exists. For
published crates, compare against the latest released version. For private or
pre-publication reviews, compare against the previous release tag or an
explicit baseline commit.

## Upstream Dependency Timing

Measure compile-cost impact when adding or materially changing foundational
dependencies. For `of_core`, use default features disabled unless a documented
integration requires otherwise:

```toml
[dependencies]
of_core = { version = "0.4.0", default-features = false }
```

Recommended timing command:

```bash
CARGO_TARGET_DIR=/tmp/of-core-timing-target cargo build --timings
```

Record the dependency version, enabled features, command output, hardware,
operating system, Rust version, and timing artifact path in release evidence.

## Packaging Order

Package verification for dependent crates resolves versioned path dependencies
through the registry after Cargo removes local `path` entries from packaged
manifests. Verify and publish shared foundational crates before downstream
crates that depend on them. For this workspace, verify `risk-core` first, then
verify `risk-pretrade`, `risk-portfolio`, and benchmark or integration crates
against the published or explicitly patched core baseline.
