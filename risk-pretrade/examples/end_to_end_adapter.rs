//! End-to-end adapter example for an order-entry integration.

use std::{error::Error, fmt};

use risk_core::{
    CurrencyId, DataQuality, EquitySpec, Instrument, InstrumentCatalog, InstrumentId, MarketPrice,
    MarketSnapshot, Notional, Price, Qty, SymbolKey, SymbolRegistry, Timestamp,
};
use risk_pretrade::{EvaluateRequest, GateAuditRecord, InMemoryAuditLog, LimitTable, PretradeGate};
use risk_pretrade::{ObservedOrderEvent, TraceContext};

fn main() -> Result<(), AdapterError> {
    let mut audit_log = InMemoryAuditLog::new();
    let (registry, catalog) = reference_data()?;
    let gate = PretradeGate::new(limits());
    let market = market();
    let incoming = AdapterOrder {
        venue: "XNYS",
        symbol: "IBM",
        qty: 5,
        current_position: 0,
        available_margin: 1_000,
        order_price: 100,
        received_at: Timestamp(10),
    };

    let limit_audit = gate.update_limits_with_audit(limits(), "file-limit-source", Timestamp(9));
    audit_log.push(GateAuditRecord::LimitChange(limit_audit));

    let (verdict, audit_record) =
        evaluate_adapter_order(&gate, &registry, &catalog, &market, incoming)?;
    let observed = ObservedOrderEvent::new(
        TraceContext {
            correlation_id: 42,
            sequence: 1,
            observed_at: incoming.received_at,
        },
        audit_record.clone(),
        gate.metrics_snapshot(),
    );
    audit_log.push(GateAuditRecord::OrderEvaluation(audit_record));

    println!("verdict={verdict:?}");
    println!("audit_records={}", audit_log.records().len());
    println!("observed_evaluations={}", observed.metrics.evaluations);

    Ok(())
}

fn evaluate_adapter_order(
    gate: &PretradeGate,
    registry: &SymbolRegistry,
    catalog: &InstrumentCatalog,
    market: &MarketSnapshot,
    order: AdapterOrder<'_>,
) -> Result<(risk_core::RiskVerdict, risk_pretrade::OrderAuditRecord), AdapterError> {
    let symbol = SymbolKey {
        venue: order.venue.to_owned(),
        symbol: order.symbol.to_owned(),
    };
    let instrument_id = registry.resolve(&symbol)?;
    let instrument = catalog
        .get(instrument_id)
        .ok_or(AdapterError::MissingInstrument(instrument_id))?;

    Ok(gate.evaluate_with_audit(EvaluateRequest {
        instrument,
        qty: Qty::new(order.qty),
        current_position: Qty::new(order.current_position),
        available_margin: Notional::new(order.available_margin),
        order_price: Price::new(order.order_price),
        market,
        now: order.received_at,
    }))
}

fn reference_data() -> Result<(SymbolRegistry, InstrumentCatalog), AdapterError> {
    let mut registry = SymbolRegistry::new();
    let mut catalog = InstrumentCatalog::new();
    let instrument = Instrument::Equity(EquitySpec {
        instrument_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
    });

    registry.register(
        SymbolKey {
            venue: "XNYS".to_owned(),
            symbol: "IBM".to_owned(),
        },
        InstrumentId(1),
    )?;
    catalog.insert(instrument)?;

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
    market.set_aggregate_notional(Notional::new(0), Timestamp(5), DataQuality::clean());
    market
}

#[derive(Debug, Clone, Copy)]
struct AdapterOrder<'a> {
    venue: &'a str,
    symbol: &'a str,
    qty: i64,
    current_position: i64,
    available_margin: i64,
    order_price: i64,
    received_at: Timestamp,
}

#[derive(Debug)]
enum AdapterError {
    RegisterSymbol(risk_core::RegisterSymbolError),
    InstrumentCatalog(risk_core::InstrumentCatalogError),
    Indeterminate(risk_core::IndeterminateReason),
    MissingInstrument(InstrumentId),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegisterSymbol(error) => write!(f, "{error}"),
            Self::InstrumentCatalog(error) => write!(f, "{error}"),
            Self::Indeterminate(reason) => write!(f, "symbol resolution failed: {reason:?}"),
            Self::MissingInstrument(instrument_id) => {
                write!(f, "missing instrument {}", instrument_id.raw())
            }
        }
    }
}

impl Error for AdapterError {}

impl From<risk_core::RegisterSymbolError> for AdapterError {
    fn from(error: risk_core::RegisterSymbolError) -> Self {
        Self::RegisterSymbol(error)
    }
}

impl From<risk_core::InstrumentCatalogError> for AdapterError {
    fn from(error: risk_core::InstrumentCatalogError) -> Self {
        Self::InstrumentCatalog(error)
    }
}

impl From<risk_core::IndeterminateReason> for AdapterError {
    fn from(reason: risk_core::IndeterminateReason) -> Self {
        Self::Indeterminate(reason)
    }
}
