## Summary

- 

## Risk Review

- [ ] I reviewed fail-closed behavior for changed paths.
- [ ] I reviewed fixed-point arithmetic, parser, schema, or market-data changes where applicable.
- [ ] I reviewed validation fixture impact.
- [ ] I reviewed documentation and migration impact.
- [ ] External reviewer approval is required before merge.

## Evidence

- [ ] `cargo fmt --all --check`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo test --workspace --examples --all-features`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`
- [ ] `cargo audit --db target/advisory-db`
- [ ] `cargo deny check`
- [ ] Benchmark or rationale attached when latency-sensitive code changed.

## Sign-Off

- Model validation impact: 
- Security review impact: 
- Operations impact: 
