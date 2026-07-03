# Validation Pack

The institutional prototype validation pack lives in `risk-pretrade/tests` and
`risk-portfolio/tests`.

## Golden Pretrade Fixtures

`risk-pretrade/tests/fixtures/pretrade_orders.csv` defines end-to-end pretrade
cases with:

- adapter-style venue and symbol fields,
- symbol-registry resolution into `InstrumentId`,
- instrument-catalog lookup,
- market and limit snapshot evaluation,
- expected `RiskVerdict` strings.

The fixture test is:

```bash
cargo test -p risk-pretrade --test golden_pretrade
```

Current scenarios cover:

- passing equity order,
- per-order notional rejection,
- position-limit rejection,
- fat-finger rejection,
- v1 option indeterminate behavior.

## Golden Portfolio Stress Fixtures

`risk-portfolio/tests/fixtures/stress_scenarios.csv` defines deterministic
portfolio stress scenarios with:

- ordered asset indexes,
- additive return shocks,
- expected weighted portfolio loss.

The fixture test is:

```bash
cargo test -p risk-portfolio --test golden_stress
```

Current scenarios cover:

- single-asset equity drawdown,
- single-asset rate shock,
- multi-asset broad risk-off shock.

## Adapter Example

`risk-pretrade/examples/end_to_end_adapter.rs` shows the expected order-entry
adapter shape:

1. receive venue/symbol/price/quantity fields,
2. resolve `SymbolKey` through `SymbolRegistry`,
3. fetch static instrument reference data from `InstrumentCatalog`,
4. evaluate with `PretradeGate`,
5. emit audit records for limit changes and order decisions.

Run it with:

```bash
cargo run -p risk-pretrade --example end_to_end_adapter
```
