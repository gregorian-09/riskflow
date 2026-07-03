# risk-bench

Benchmark harnesses for Riskflow pretrade evaluation.

`risk-bench` is separate from production crates so benchmark dependencies stay
out of normal library builds.

Read the full guide:

- [risk-bench crate guide](https://github.com/gregorian-09/riskflow/blob/master/docs/crates/risk-bench.md)
- [Benchmark methodology](https://github.com/gregorian-09/riskflow/blob/master/docs/benchmarks.md)
- [Benchmark matrix](https://github.com/gregorian-09/riskflow/blob/master/docs/benchmark_matrix.md)

## What It Measures

- steady-read `PretradeGate::evaluate`,
- evaluation while limit snapshots are replaced,
- median latency,
- p99.9 latency.

## Run

```bash
cargo run -p risk-bench --release -- --iterations 5000
```
