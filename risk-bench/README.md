# risk-bench

Benchmark harnesses for Riskflow pretrade evaluation.

`risk-bench` is separate from production crates so benchmark dependencies stay
out of normal library builds.

Primary documentation:

- [risk-bench crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-bench.md)
- [Benchmark fixture example](https://github.com/gregorian-09/riskflow/blob/master/risk-bench/examples/benchmark_fixture.rs)
- [Benchmark methodology](https://github.com/gregorian-09/riskflow/blob/master/docs/benchmarks.md)
- [Benchmark matrix](https://github.com/gregorian-09/riskflow/blob/master/docs/benchmark_matrix.md)
- [Release governance](https://github.com/gregorian-09/riskflow/blob/master/docs/release_governance.md)

## Public API Inventory

`risk-bench` is an executable benchmark crate rather than a library API. Its
public surfaces are:

- `cargo run -p risk-bench --release -- --iterations <N>` for release-style
  latency smoke reports.
- `cargo bench -p risk-bench --bench evaluate` for Criterion development
  benchmarks.
- `cargo run -p risk-bench --example benchmark_fixture` for checking the
  deterministic pretrade fixture used by the benchmark.

## What It Measures

- steady-read `PretradeGate::evaluate`,
- evaluation while limit snapshots are replaced,
- median latency,
- p99.9 latency.

## Output Semantics

- `steady_read.*` measures repeated `PretradeGate::evaluate` calls with a
  stable limit snapshot.
- `contended_updates.*` measures evaluation while another thread repeatedly
  replaces the limit snapshot.
- `median_ns` is the 50th percentile sample latency in nanoseconds.
- `p99_9_ns` is the 99.9th percentile sample latency in nanoseconds.

## Choosing The Right Benchmark

| Need | Use |
|---|---|
| Quick local harness check | `cargo run -p risk-bench --release -- --iterations 5000` |
| Release evidence smoke | `cargo run -p risk-bench --release -- --iterations 50000` |
| Development comparison while editing code | `cargo bench -p risk-bench --bench evaluate` |
| Verify the shared fixture verdict | `cargo run -p risk-bench --example benchmark_fixture` |

```mermaid
flowchart LR
    fixture[Deterministic pretrade fixture] --> steady[Steady evaluate loop]
    fixture --> contended[Evaluate during limit replacement]
    steady --> report[Latency report]
    contended --> report
    report --> matrix[Benchmark matrix evidence]
```

## Command-Line Smoke

```bash
cargo run -p risk-bench --release -- --iterations 5000
```

Expected output shape:

```text
pretrade evaluate latency report
iterations: 5000
steady_read.median_ns: ...
steady_read.p99_9_ns: ...
contended_updates.median_ns: ...
contended_updates.p99_9_ns: ...
```

## Criterion Bench

```bash
cargo bench -p risk-bench --bench evaluate -- --test
```

Use Criterion during development when comparing code changes locally. Use the
command-line smoke for release evidence because it emits compact text that CI
can archive.

## Production Benchmark Runner

The GitHub benchmark workflow expects a self-hosted runner labeled
`risk-prod-bench` when collecting production-like evidence. A development
machine or WSL host can validate the workflow wiring, but release claims should
come from the documented benchmark machine recorded in
[Benchmark Matrix](https://github.com/gregorian-09/riskflow/blob/master/docs/benchmark_matrix.md).

Each benchmark record should include:

- CPU model and core count,
- operating system and kernel,
- Rust version,
- iteration count,
- median and p99.9 latency for steady reads,
- median and p99.9 latency during contended limit updates,
- whether the host is development, CI, or production-like.

## Real-World Use Cases

### Release evidence

Run the benchmark CLI on approved hardware, archive the command output, and
copy the results into the benchmark matrix with hardware and toolchain details.

### Performance regression review

Use Criterion while changing hot-path checks, limit storage, or market snapshot
lookup behavior. Compare before and after results on the same machine.

### Runner wiring check

Run the fixture example and short smoke command when validating a new CI or
self-hosted benchmark runner.

## Read Next

- [Full crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-bench.md) for fixture construction and reporting rules.
- [Benchmark methodology](https://github.com/gregorian-09/riskflow/blob/master/docs/benchmarks.md) for reproducible run conditions.
- [Release governance](https://github.com/gregorian-09/riskflow/blob/master/docs/release_governance.md) for required evidence workflows.

## Verify

```bash
cargo run -p risk-bench --release -- --iterations 5000
cargo bench -p risk-bench --bench evaluate -- --test
cargo run -p risk-bench --example benchmark_fixture
```
