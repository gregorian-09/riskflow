# `risk-bench`

`risk-bench` contains benchmark harnesses and release smoke commands for the
pretrade gate. It is separated from production crates so benchmark-only
dependencies do not leak into normal builds.

## What It Measures

```mermaid
flowchart LR
    steady[steady read path] --> eval[PretradeGate::evaluate]
    updates[limit table replacements] --> eval
    eval --> median[median latency]
    eval --> p999[p99.9 latency]
```

Benchmarks cover:

- steady-read pretrade evaluation,
- evaluation while limit snapshots are repeatedly replaced,
- command-line smoke reporting for release evidence.

## Public Surface Inventory

`risk-bench` exposes executable surfaces:

- benchmark CLI: `cargo run -p risk-bench --release -- --iterations <N>`,
- Criterion harness: `cargo bench -p risk-bench --bench evaluate`,
- deterministic fixture example: `cargo run -p risk-bench --example benchmark_fixture`.

## Output Semantics

- `steady_read.median_ns` and `steady_read.p99_9_ns` describe evaluation
  latency with a stable limit snapshot.
- `contended_updates.median_ns` and `contended_updates.p99_9_ns` describe
  evaluation latency while another thread replaces limit snapshots.
- Results are environment-specific. Production claims require target hardware
  evidence in the benchmark matrix.

## Choosing The Right Command

| Need | Command |
|---|---|
| Verify the executable path | `cargo run -p risk-bench --release -- --iterations 5000` |
| Capture release smoke evidence | `cargo run -p risk-bench --release -- --iterations 50000` |
| Compare code changes locally | `cargo bench -p risk-bench --bench evaluate` |
| Verify fixture verdict | `cargo run -p risk-bench --example benchmark_fixture` |

## Command-Line Smoke

Short local smoke:

```bash
cargo run -p risk-bench --release -- --iterations 5000
```

Release evidence smoke:

```bash
cargo run -p risk-bench --release -- --iterations 50000
```

Expected output shape:

```text
pretrade evaluate latency report
iterations: 50000
steady_read.median_ns: ...
steady_read.p99_9_ns: ...
contended_updates.median_ns: ...
contended_updates.p99_9_ns: ...
```

## Criterion Bench

```bash
cargo bench -p risk-bench --bench evaluate -- --test
cargo run -p risk-bench --example benchmark_fixture
```

The Criterion harness is useful during development. The command-line smoke is
more useful for release evidence because it emits a compact text report that CI
can archive.

## Benchmark Construction

The harness builds:

- one equity instrument,
- one limit table,
- one trusted market snapshot,
- one valid order request.

The contended-update path alternates limit-table versions while evaluating.
This is not a full exchange simulation; it isolates the overhead of the gate
and the chosen limit snapshot strategy.

## Reporting Rules

Record production-like results in [Benchmark Matrix](../benchmark_matrix.md).
Rows must include:

- hardware,
- operating system and kernel,
- Rust version,
- iteration count,
- median latency,
- p99.9 latency,
- whether the runner was a development, CI, or production-like host.

Development-machine results are benchmark-harness checks only. Treat latency
claims as production-like only when they come from documented target hardware.

## Real-World Use Cases

### Release benchmark evidence

Run the release smoke command on approved hardware and archive both environment
metadata and latency output.

### Hot-path regression review

Use Criterion to compare before and after changes to checks, limit storage, or
market snapshot lookup behavior.

### CI runner validation

Run the short smoke command and fixture example when attaching a new benchmark
runner.

## Maintainer Guidance

Add a benchmark when a public runtime characteristic changes:

- a new pretrade check enters the hot path,
- limit storage changes,
- market snapshot lookup behavior changes,
- observability changes add measurable overhead.

Keep benchmark fixtures deterministic and documented.
