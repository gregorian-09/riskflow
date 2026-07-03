//! Adversarial pretrade validation scenarios.

use risk_core::{
    CurrencyId, DataQuality, DataQualityFlags, EquitySpec, IndeterminateReason, Instrument,
    InstrumentId, MarketPrice, MarketSnapshot, Notional, Price, Qty, RejectReason,
    RiskDataQualityFlags, RiskVerdict, Timestamp,
};
use risk_pretrade::{EvaluateRequest, LimitTable, PretradeGate};

#[test]
fn minimum_quantity_overflow_fails_closed() {
    let gate = PretradeGate::new(limits());
    let market = market_with_price(MarketPrice::clean(Price::new(100), Timestamp(10)));

    let verdict = gate.evaluate(request(&market, Qty::new(i64::MIN), Price::new(100)));

    assert_eq!(
        verdict,
        RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow)
    );
}

#[test]
fn degraded_upstream_feed_fails_closed() {
    let gate = PretradeGate::new(limits());
    let market = market_with_price(MarketPrice {
        price: Price::new(100),
        observed_at: Timestamp(10),
        quality: DataQuality::from_upstream(DataQualityFlags::ADAPTER_DEGRADED),
    });

    let verdict = gate.evaluate(request(&market, Qty::new(5), Price::new(100)));

    assert_eq!(
        verdict,
        RiskVerdict::Indeterminate(IndeterminateReason::BadDataQuality)
    );
}

#[test]
fn stale_aggregate_quality_fails_closed() {
    let gate = PretradeGate::new(limits());
    let mut market = market_with_price(MarketPrice::clean(Price::new(100), Timestamp(10)));
    market.set_aggregate_notional(
        Notional::new(0),
        Timestamp(10),
        DataQuality {
            upstream_flags: DataQualityFlags::NONE,
            risk_flags: RiskDataQualityFlags::STALE_AGGREGATE,
        },
    );

    let verdict = gate.evaluate(request(&market, Qty::new(5), Price::new(100)));

    assert_eq!(
        verdict,
        RiskVerdict::Indeterminate(IndeterminateReason::BadDataQuality)
    );
}

#[test]
fn disabled_trading_rejects_even_with_valid_market() {
    let gate = PretradeGate::new(limits());
    gate.disable_trading_with_audit("ops", Timestamp(9));
    let market = market_with_price(MarketPrice::clean(Price::new(100), Timestamp(10)));

    let verdict = gate.evaluate(request(&market, Qty::new(5), Price::new(100)));

    assert_eq!(verdict, RiskVerdict::Reject(RejectReason::TradingDisabled));
}

fn request(market: &MarketSnapshot, qty: Qty, order_price: Price) -> EvaluateRequest<'_> {
    EvaluateRequest {
        instrument: Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        }),
        qty,
        current_position: Qty::new(0),
        available_margin: Notional::new(1_000),
        order_price,
        market,
        now: Timestamp(10),
    }
}

fn limits() -> LimitTable {
    let mut limits = LimitTable::new();
    limits.set_per_order_notional(InstrumentId(1), Notional::new(i64::MAX));
    limits.set_aggregate_notional(Notional::new(i64::MAX));
    limits.set_max_abs_position(InstrumentId(1), Qty::new(i64::MAX));
    limits.set_fat_finger_band_bps(InstrumentId(1), 500);
    limits.set_initial_margin_per_unit(InstrumentId(1), Notional::new(1));
    limits
}

fn market_with_price(price: MarketPrice) -> MarketSnapshot {
    let mut market = MarketSnapshot::new(10, 10, 10);
    market.insert_price(InstrumentId(1), price);
    market.set_aggregate_notional(Notional::new(0), Timestamp(10), DataQuality::clean());
    market
}
