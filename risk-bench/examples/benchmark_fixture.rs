//! Construct and evaluate the deterministic benchmark fixture.

use risk_core::{
    CurrencyId, DataQuality, EquitySpec, Instrument, InstrumentId, MarketPrice, MarketSnapshot,
    Notional, Price, Qty, Timestamp,
};
use risk_pretrade::{EvaluateRequest, LimitTable, PretradeGate};

fn main() {
    let instrument = Instrument::Equity(EquitySpec {
        instrument_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
    });
    let gate = PretradeGate::new(limits());
    let market = market();

    let verdict = gate.evaluate(EvaluateRequest {
        instrument,
        qty: Qty::new(5),
        current_position: Qty::new(0),
        available_margin: Notional::new(1_000),
        order_price: Price::new(100),
        market: &market,
        now: Timestamp(10),
    });

    println!("benchmark_fixture_verdict={verdict:?}");
}

fn limits() -> LimitTable {
    let mut limits = LimitTable::new();
    limits.set_per_order_notional(InstrumentId(1), Notional::new(1_000));
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
