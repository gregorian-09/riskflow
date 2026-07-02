//! Pretrade gate evaluation benchmarks.

#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use risk_core::{
    CurrencyId, DataQuality, EquitySpec, Instrument, InstrumentId, MarketPrice, MarketSnapshot,
    Notional, Price, Qty, Timestamp,
};
use risk_pretrade::{EvaluateRequest, LimitTable, PretradeGate};

fn evaluate_steady_read(c: &mut Criterion) {
    let instrument = equity();
    let gate = PretradeGate::new(limits(1_000));
    let market = market();

    c.bench_function("pretrade_evaluate_steady_read", |b| {
        b.iter(|| {
            let verdict = gate.evaluate(EvaluateRequest {
                instrument,
                qty: Qty::new(5),
                current_position: Qty::new(0),
                available_margin: Notional::new(1_000),
                order_price: Price::new(100),
                market: &market,
                now: Timestamp(10),
            });
            criterion::black_box(verdict);
        });
    });
}

fn evaluate_with_limit_swaps(c: &mut Criterion) {
    let instrument = equity();
    let gate = PretradeGate::new(limits(1_000));
    let market = market();
    let mut version = 0_i64;

    c.bench_function("pretrade_evaluate_with_limit_swaps", |b| {
        b.iter(|| {
            version += 1;
            gate.update_limits(limits(1_000 + version % 2));
            let verdict = gate.evaluate(EvaluateRequest {
                instrument,
                qty: Qty::new(5),
                current_position: Qty::new(0),
                available_margin: Notional::new(1_000),
                order_price: Price::new(100),
                market: &market,
                now: Timestamp(10),
            });
            criterion::black_box(verdict);
        });
    });
}

fn limits(per_order: i64) -> LimitTable {
    let mut limits = LimitTable::new();
    limits.set_per_order_notional(InstrumentId(1), Notional::new(per_order));
    limits.set_aggregate_notional(Notional::new(10_000));
    limits.set_max_abs_position(InstrumentId(1), Qty::new(100));
    limits.set_fat_finger_band_bps(InstrumentId(1), 500);
    limits.set_initial_margin_per_unit(InstrumentId(1), Notional::new(10));
    limits
}

fn market() -> MarketSnapshot {
    let mut market = MarketSnapshot::new(10, 10, 10);
    market.insert_price(
        InstrumentId(1),
        MarketPrice::clean(Price::new(100), Timestamp(5)),
    );
    market.set_aggregate_notional(Notional::new(0), Timestamp(5), DataQuality::clean());
    market
}

const fn equity() -> Instrument {
    Instrument::Equity(EquitySpec {
        instrument_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
    })
}

criterion_group!(benches, evaluate_steady_read, evaluate_with_limit_swaps);
criterion_main!(benches);
