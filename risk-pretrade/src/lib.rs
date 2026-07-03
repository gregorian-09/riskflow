//! Synchronous pretrade risk gate.
//!
//! `risk-pretrade` evaluates fixed-point order risk against immutable limit
//! snapshots. The read path uses a single `ArcSwap` strategy for limit-table
//! replacement; source flexibility lives outside the hot path through
//! [`LimitSource`].
//!
//! # Failure Model
//!
//! Any missing, stale, low-quality, unsupported, or overflowing input returns
//! an indeterminate verdict instead of passing the order.
//!
//! # Minimal Evaluation
//!
//! ```
//! use risk_core::{
//!     CurrencyId, DataQuality, EquitySpec, Instrument, InstrumentId, MarketPrice,
//!     MarketSnapshot, Notional, Price, Qty, Timestamp,
//! };
//! use risk_pretrade::{EvaluateRequest, LimitTable, PretradeGate};
//!
//! let instrument = Instrument::Equity(EquitySpec {
//!     instrument_id: InstrumentId(1),
//!     settlement_currency: CurrencyId(840),
//! });
//!
//! let mut limits = LimitTable::new();
//! limits.set_per_order_notional(InstrumentId(1), Notional::new(1_000));
//! limits.set_aggregate_notional(Notional::new(10_000));
//! limits.set_max_abs_position(InstrumentId(1), Qty::new(100));
//! limits.set_fat_finger_band_bps(InstrumentId(1), 500);
//! limits.set_initial_margin_per_unit(InstrumentId(1), Notional::new(10));
//!
//! let gate = PretradeGate::new(limits);
//!
//! let mut market = MarketSnapshot::new(10, 10, 10);
//! market.insert_price(
//!     InstrumentId(1),
//!     MarketPrice::clean(Price::new(100), Timestamp(5)),
//! );
//! market.set_aggregate_notional(Notional::new(0), Timestamp(5), DataQuality::clean());
//!
//! let verdict = gate.evaluate(EvaluateRequest {
//!     instrument,
//!     qty: Qty::new(5),
//!     current_position: Qty::new(0),
//!     available_margin: Notional::new(1_000),
//!     order_price: Price::new(100),
//!     market: &market,
//!     now: Timestamp(10),
//! });
//!
//! assert!(verdict.is_pass());
//! ```
//!
//! # Audit And Metrics
//!
//! `evaluate_with_audit` returns the decision and the corresponding
//! [`OrderAuditRecord`]. [`PretradeGate::metrics_snapshot`] exposes counters
//! that adapters can export through their own logging, tracing, metrics, or
//! telemetry stack.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audit;
pub mod checks;
pub mod gate;
pub mod limit_source;
pub mod observability;

pub use audit::{
    GateAuditRecord, InMemoryAuditLog, LimitChangeAuditRecord, OrderAuditRecord,
    TradingStateAuditRecord,
};
pub use gate::{EvaluateRequest, LimitTable, PretradeGate};
pub use limit_source::{
    FileLimitSource, LimitSource, ParseLimitTableError, ParseLimitTableErrorKind,
    StaticLimitSource, parse_limit_table,
};
pub use observability::{
    AlertSeverity, GateMetrics, GateMetricsSnapshot, ObservedLimitChangeEvent, ObservedOrderEvent,
    ObservedTradingStateEvent, PretradeAlert, TraceContext,
};
