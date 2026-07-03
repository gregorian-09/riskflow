# Benchmark Matrix

The risk gate must be benchmarked on real target hardware before production
adoption. Development-machine numbers are useful for checking the benchmark
harness, but they are not sufficient for institutional latency claims.

## Required Command

```bash
cargo run -p risk-bench --release -- --iterations 50000
```

## Published Matrix

| Environment | CPU | OS | Rust | Iterations | Steady Median ns | Steady p99.9 ns | Contended Median ns | Contended p99.9 ns | Status |
|---|---|---|---|---|---|---|---|---|---|
| _No production-like rows published yet_ | _TBD_ | _TBD_ | _TBD_ | _TBD_ | _TBD_ | _TBD_ | _TBD_ | Awaiting release evidence |

## Required Production Rows

Add rows before production approval for:

- target trading server,
- warm standby server,
- CI reference runner,
- disaster-recovery environment.

Each row must record power profile, CPU governor, kernel, Rust version, commit
hash, and whether simultaneous market-data and limit-update load was present.
