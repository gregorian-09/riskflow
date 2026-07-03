//! Shared types for the Risk workspace.
//!
//! `risk-core` owns the fixed-point identifiers, instruments, positions,
//! market snapshots, and verdict types consumed by latency-sensitive and
//! offline risk crates.
//!
//! # Scope
//!
//! v1 covers equities, spot FX, spot crypto, futures, and perpetual swaps.
//! Options are present as taxonomy placeholders but return
//! [`RiskWeight::Indeterminate`] until an isolated options crate is available.
//!
//! # Failure Model
//!
//! Missing prices, stale snapshots, bad upstream data quality, arithmetic
//! overflow, and unsupported instruments are represented explicitly with
//! [`IndeterminateReason`]. Callers should treat indeterminate risk as
//! fail-closed.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod currency;
pub mod instrument;
pub mod market;
pub mod position;
pub mod schema;
pub mod symbol;
pub mod types;
pub mod verdict;

pub use currency::{CurrencyId, CurrencyPair};
pub use instrument::{
    AssetClass, CryptoSpotSpec, CurrencySet, EquitySpec, FutureSpec, FxSpec, Instrument,
    InstrumentCatalog, InstrumentCatalogError, OptionSpec, PerpSpec, RiskExposure,
};
pub use market::{
    DataQuality, DataQualityFlags, MarketPrice, MarketSnapshot, RiskDataQualityFlags,
};
pub use position::{Funding, MarginState, Position};
pub use schema::{
    AUDIT_RECORD_SCHEMA, INSTRUMENT_REFERENCE_SCHEMA, LIMIT_TABLE_SCHEMA, MARKET_SNAPSHOT_SCHEMA,
    PORTFOLIO_VALIDATION_SCHEMA, SchemaDescriptor, SchemaRecordKind, SchemaVersion, current_schema,
};
pub use symbol::{RegisterSymbolError, SymbolKey, SymbolRegistry};
pub use types::{InstrumentId, Notional, Price, Qty, Timestamp};
pub use verdict::{IndeterminateReason, RejectReason, RiskVerdict, RiskWeight};
