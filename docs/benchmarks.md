# Benchmark Report

This file records the reproducible benchmark method for the pretrade gate.
Update it with the target machine details whenever publishing release numbers.

## Command

```bash
cargo run -p risk-bench --release -- --iterations 50000
```

## Required Fields

- Hardware: record CPU model, core count, memory, and power profile.
- Operating system: record distribution/version and kernel.
- Rust toolchain: record `rustc --version`.
- Build profile: `--release`.
- Iterations: record the `--iterations` value.
- Results: record median and p99.9 latency for steady reads and contended
  limit updates.

## Local Smoke Result

This short run verifies the report path and output shape; it is not a published
hardware claim.

```text
iterations: 5000
steady_read.median_ns: 100
steady_read.p99_9_ns: 200
contended_updates.median_ns: 200
contended_updates.p99_9_ns: 600
```
