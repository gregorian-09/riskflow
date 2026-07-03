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

#![forbid(unsafe_code)]
#![warn(missing_docs)]

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
