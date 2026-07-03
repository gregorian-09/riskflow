# `risk-bench`

`risk-bench` contains benchmark harnesses and release smoke commands.

## Responsibilities

- Measure steady-read pretrade evaluation latency.
- Measure evaluation under repeated limit table replacement.
- Provide a short benchmark smoke command for release evidence.

## Commands

Smoke run:

```bash
cargo run -p risk-bench --release -- --iterations 5000
```

Release-style run:

```bash
cargo run -p risk-bench --release -- --iterations 50000
```

Criterion bench:

```bash
cargo bench -p risk-bench --bench evaluate -- --test
```

## Reporting

Record results in [Benchmark Matrix](../benchmark_matrix.md) when running on
approved hardware. Local WSL or laptop results are development baselines only.
