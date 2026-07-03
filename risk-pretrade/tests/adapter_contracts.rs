//! Adapter contract tests for external order and market-data boundaries.

use risk_core::{
    CurrencyId, DataQuality, EquitySpec, IndeterminateReason, Instrument, InstrumentCatalog,
    InstrumentId, MarketPrice, MarketSnapshot, Notional, Price, Qty, RiskVerdict, SymbolKey,
    SymbolRegistry, Timestamp,
};
use risk_pretrade::{EvaluateRequest, LimitTable, PretradeGate};

#[test]
fn orderflow_symbol_and_fix_order_adapter_path_passes() {
    let (registry, catalog) = reference_data();
    let gate = PretradeGate::new(limits());
    let market = tickbar_market_snapshot(Timestamp(9));
    let order = FixLikeOrder {
        venue: "XNYS",
        symbol: "IBM",
        qty: 5,
        current_position: 0,
        available_margin: 1_000,
        limit_price: 100,
        received_at: Timestamp(10),
    };

    let verdict = evaluate_fix_like_order(&gate, &registry, &catalog, &market, order);

    assert_eq!(verdict, RiskVerdict::Pass);
}

#[test]
fn unknown_orderflow_symbol_fails_closed_before_gate() {
    let (registry, catalog) = reference_data();
    let gate = PretradeGate::new(limits());
    let market = tickbar_market_snapshot(Timestamp(9));
    let order = FixLikeOrder {
        venue: "XNAS",
        symbol: "MSFT",
        qty: 5,
        current_position: 0,
        available_margin: 1_000,
        limit_price: 100,
        received_at: Timestamp(10),
    };

    let verdict = evaluate_fix_like_order(&gate, &registry, &catalog, &market, order);

    assert_eq!(
        verdict,
        RiskVerdict::Indeterminate(IndeterminateReason::UnknownSymbol)
    );
}

#[test]
fn stale_tickbar_market_snapshot_fails_closed() {
    let (registry, catalog) = reference_data();
    let gate = PretradeGate::new(limits());
    let market = tickbar_market_snapshot(Timestamp(1));
    let order = FixLikeOrder {
        venue: "XNYS",
        symbol: "IBM",
        qty: 5,
        current_position: 0,
        available_margin: 1_000,
        limit_price: 100,
        received_at: Timestamp(20),
    };

    let verdict = evaluate_fix_like_order(&gate, &registry, &catalog, &market, order);

    assert_eq!(
        verdict,
        RiskVerdict::Indeterminate(IndeterminateReason::StalePrice)
    );
}

fn evaluate_fix_like_order(
    gate: &PretradeGate,
    registry: &SymbolRegistry,
    catalog: &InstrumentCatalog,
    market: &MarketSnapshot,
    order: FixLikeOrder<'_>,
) -> RiskVerdict {
    let symbol = SymbolKey {
        venue: order.venue.to_owned(),
        symbol: order.symbol.to_owned(),
    };
    let instrument_id = match registry.resolve(&symbol) {
        Ok(instrument_id) => instrument_id,
        Err(reason) => return RiskVerdict::Indeterminate(reason),
    };
    let Some(instrument) = catalog.get(instrument_id) else {
        return RiskVerdict::Reject(risk_core::RejectReason::UnknownInstrument);
    };

    gate.evaluate(EvaluateRequest {
        instrument,
        qty: Qty::new(order.qty),
        current_position: Qty::new(order.current_position),
        available_margin: Notional::new(order.available_margin),
        order_price: Price::new(order.limit_price),
        market,
        now: order.received_at,
    })
}

fn reference_data() -> (SymbolRegistry, InstrumentCatalog) {
    let mut registry = SymbolRegistry::new();
    let mut catalog = InstrumentCatalog::new();
    let instrument = Instrument::Equity(EquitySpec {
        instrument_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
    });

    registry
        .register(
            SymbolKey {
                venue: "XNYS".to_owned(),
                symbol: "IBM".to_owned(),
            },
            InstrumentId(1),
        )
        .unwrap();
    catalog.insert(instrument).unwrap();

    (registry, catalog)
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

fn tickbar_market_snapshot(observed_at: Timestamp) -> MarketSnapshot {
    let mut market = MarketSnapshot::new(10, 10, 10);
    market.insert_price(
        InstrumentId(1),
        MarketPrice::clean(Price::new(100), observed_at),
    );
    market.set_aggregate_notional(Notional::new(0), observed_at, DataQuality::clean());
    market
}

#[derive(Debug, Clone, Copy)]
struct FixLikeOrder<'a> {
    venue: &'a str,
    symbol: &'a str,
    qty: i64,
    current_position: i64,
    available_margin: i64,
    limit_price: i64,
    received_at: Timestamp,
}
