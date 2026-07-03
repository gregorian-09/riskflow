# Benchmark Methodology

This file records the reproducible benchmark method for the pretrade gate.
Use it to produce release evidence and benchmark-matrix rows.

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

## Report Template

Record benchmark evidence in this shape:

- Hardware:
- Operating system:
- Rust toolchain:
- Build profile: `--release`.
- Commit:
- Runner class: development, CI reference, or production-like.
- Concurrent load:

```text
iterations: 50000
steady_read.median_ns: <value>
steady_read.p99_9_ns: <value>
contended_updates.median_ns: <value>
contended_updates.p99_9_ns: <value>
```

Development-machine results can validate the benchmark harness, but production
latency claims require production-like hardware and a recorded benchmark-matrix
row.
