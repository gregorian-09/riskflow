//! Build reference data and trusted market data with `risk-core`.

use risk_core::{
    CurrencyId, DataQuality, EquitySpec, Instrument, InstrumentCatalog, InstrumentId, MarketPrice,
    MarketSnapshot, Notional, Price, SymbolKey, SymbolRegistry, Timestamp,
};

fn main() {
    let symbol = SymbolKey {
        venue: "XNYS".to_owned(),
        symbol: "IBM".to_owned(),
    };
    let instrument = Instrument::Equity(EquitySpec {
        instrument_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
    });

    let mut registry = SymbolRegistry::new();
    registry
        .register(symbol.clone(), InstrumentId(1))
        .expect("fixture symbol should register");

    let mut catalog = InstrumentCatalog::new();
    catalog
        .insert(instrument)
        .expect("fixture instrument should insert");

    let resolved = registry
        .resolve(&symbol)
        .expect("fixture symbol should resolve");
    let catalog_instrument = catalog
        .get(resolved)
        .expect("fixture instrument should exist");

    let mut market = MarketSnapshot::new(10, 10, 10);
    market.insert_price(resolved, MarketPrice::clean(Price::new(100), Timestamp(5)));
    market.set_aggregate_notional(Notional::new(0), Timestamp(5), DataQuality::clean());

    let trusted_price = market
        .trusted_price(resolved, Timestamp(10))
        .expect("fixture price should be trusted");

    println!("resolved_id={}", resolved.raw());
    println!("instrument={catalog_instrument:?}");
    println!("trusted_price={}", trusted_price.raw());
}
