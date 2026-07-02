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

## Local Report

This local run verifies the report path and gives a baseline for development
machines. Treat it as environment-specific, not as a universal latency claim.

- Hardware: Intel(R) Core(TM) i7-8650U CPU @ 1.90GHz, 4 cores / 8 threads.
- Operating system: Linux 6.6.87.2-microsoft-standard-WSL2 x86_64.
- Rust toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`.
- Build profile: `--release`.

```text
iterations: 50000
steady_read.median_ns: 200
steady_read.p99_9_ns: 1800
contended_updates.median_ns: 300
contended_updates.p99_9_ns: 16300
```
