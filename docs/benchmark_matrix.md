# Benchmark Matrix

The risk gate must be benchmarked on real target hardware before production
adoption. Local WSL numbers are useful for development but are not sufficient
for institutional latency claims.

## Required Command

```bash
cargo run -p risk-bench --release -- --iterations 50000
```

## Current Matrix

| Environment | CPU | OS | Rust | Iterations | Steady Median ns | Steady p99.9 ns | Contended Median ns | Contended p99.9 ns | Status |
|---|---|---|---|---:|---:|---:|---:|---:|---|
| Local WSL2 development | Intel i7-8650U, 4C/8T | Linux 6.6.87.2-microsoft-standard-WSL2 | 1.95.0 | 50000 | 200 | 1800 | 300 | 16300 | Development baseline |

## Required Production Rows

Add rows before production approval for:

- target trading server,
- warm standby server,
- CI reference runner,
- disaster-recovery environment.

Each row must record power profile, CPU governor, kernel, Rust version, commit
hash, and whether simultaneous market-data and limit-update load was present.
