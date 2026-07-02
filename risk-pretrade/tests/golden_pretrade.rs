//! Golden pretrade validation fixtures.

use std::{error::Error, fmt};

use risk_core::{
    CurrencyId, DataQuality, EquitySpec, Instrument, InstrumentCatalog, InstrumentId, MarketPrice,
    MarketSnapshot, Notional, OptionSpec, Price, Qty, SymbolKey, SymbolRegistry, Timestamp,
};
use risk_pretrade::{EvaluateRequest, LimitTable, PretradeGate};

const FIXTURES: &str = include_str!("fixtures/pretrade_orders.csv");

#[test]
fn golden_pretrade_verdicts_match() -> Result<(), FixtureError> {
    let (registry, catalog) = reference_data()?;
    let gate = PretradeGate::new(limits());
    let market = market();

    for row in parse_fixtures()? {
        let symbol = SymbolKey {
            venue: row.venue,
            symbol: row.symbol,
        };
        let instrument_id = registry.resolve(&symbol)?;
        let instrument = catalog
            .get(instrument_id)
            .ok_or(FixtureError::MissingInstrument(instrument_id))?;

        let verdict = gate.evaluate(EvaluateRequest {
            instrument,
            qty: Qty::new(row.qty),
            current_position: Qty::new(row.current_position),
            available_margin: Notional::new(row.available_margin),
            order_price: Price::new(row.order_price),
            market: &market,
            now: Timestamp(10),
        });

        assert_eq!(format!("{verdict:?}"), row.expected, "{}", row.scenario);
    }

    Ok(())
}

fn reference_data() -> Result<(SymbolRegistry, InstrumentCatalog), FixtureError> {
    let mut registry = SymbolRegistry::new();
    let mut catalog = InstrumentCatalog::new();
    let equity = Instrument::Equity(EquitySpec {
        instrument_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
    });
    let option = Instrument::Option(OptionSpec {
        instrument_id: InstrumentId(2),
        underlying_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
        expiry: Timestamp(1_000),
    });

    registry.register(
        SymbolKey {
            venue: "XNYS".to_owned(),
            symbol: "IBM".to_owned(),
        },
        InstrumentId(1),
    )?;
    registry.register(
        SymbolKey {
            venue: "OPRA".to_owned(),
            symbol: "IBM240119C00100000".to_owned(),
        },
        InstrumentId(2),
    )?;
    catalog.insert(equity)?;
    catalog.insert(option)?;

    Ok((registry, catalog))
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
    market.insert_price(
        InstrumentId(2),
        MarketPrice::clean(Price::new(100), Timestamp(5)),
    );
    market.set_aggregate_notional(Notional::new(0), Timestamp(5), DataQuality::clean());
    market
}

fn parse_fixtures() -> Result<Vec<FixtureRow>, FixtureError> {
    let mut rows = Vec::new();

    for (index, line) in FIXTURES.lines().enumerate().skip(1) {
        let fields = line.split(',').collect::<Vec<_>>();
        let [
            scenario,
            venue,
            symbol,
            qty,
            current_position,
            available_margin,
            order_price,
            expected,
        ] = fields.as_slice()
        else {
            return Err(FixtureError::InvalidRow(index + 1));
        };

        rows.push(FixtureRow {
            scenario: (*scenario).to_owned(),
            venue: (*venue).to_owned(),
            symbol: (*symbol).to_owned(),
            qty: parse_i64(index + 1, qty)?,
            current_position: parse_i64(index + 1, current_position)?,
            available_margin: parse_i64(index + 1, available_margin)?,
            order_price: parse_i64(index + 1, order_price)?,
            expected: (*expected).to_owned(),
        });
    }

    Ok(rows)
}

fn parse_i64(line: usize, value: &str) -> Result<i64, FixtureError> {
    value
        .parse()
        .map_err(|_| FixtureError::InvalidInteger(line, value.to_owned()))
}

#[derive(Debug)]
struct FixtureRow {
    scenario: String,
    venue: String,
    symbol: String,
    qty: i64,
    current_position: i64,
    available_margin: i64,
    order_price: i64,
    expected: String,
}

#[derive(Debug)]
enum FixtureError {
    InvalidRow(usize),
    InvalidInteger(usize, String),
    RegisterSymbol(risk_core::RegisterSymbolError),
    InstrumentCatalog(risk_core::InstrumentCatalogError),
    Indeterminate(risk_core::IndeterminateReason),
    MissingInstrument(InstrumentId),
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRow(line) => write!(f, "invalid fixture row on line {line}"),
            Self::InvalidInteger(line, value) => {
                write!(f, "invalid integer `{value}` on line {line}")
            }
            Self::RegisterSymbol(error) => write!(f, "{error}"),
            Self::InstrumentCatalog(error) => write!(f, "{error}"),
            Self::Indeterminate(reason) => write!(f, "symbol resolution failed: {reason:?}"),
            Self::MissingInstrument(instrument_id) => {
                write!(f, "missing instrument {}", instrument_id.raw())
            }
        }
    }
}

impl Error for FixtureError {}

impl From<risk_core::RegisterSymbolError> for FixtureError {
    fn from(error: risk_core::RegisterSymbolError) -> Self {
        Self::RegisterSymbol(error)
    }
}

impl From<risk_core::InstrumentCatalogError> for FixtureError {
    fn from(error: risk_core::InstrumentCatalogError) -> Self {
        Self::InstrumentCatalog(error)
    }
}

impl From<risk_core::IndeterminateReason> for FixtureError {
    fn from(reason: risk_core::IndeterminateReason) -> Self {
        Self::Indeterminate(reason)
    }
}
