//! Shared domain contracts for Riskflow.
//!
//! `risk-core` owns the fixed-point identifiers, instrument reference types,
//! positions, market snapshots, schema descriptors, and verdict types consumed
//! by latency-sensitive pretrade checks and offline risk analytics.
//!
//! # Scope
//!
//! v1 covers equities, spot FX, spot crypto, futures, and perpetual swaps.
//! Options are represented as unsupported v1 taxonomy and return
//! [`RiskWeight::Indeterminate`] rather than a pass.
//!
//! # Failure Model
//!
//! Missing prices, stale snapshots, bad upstream data quality, arithmetic
//! overflow, and unsupported instruments are represented explicitly with
//! [`IndeterminateReason`]. Callers should treat indeterminate risk as
//! fail-closed.
//!
//! # Symbol And Instrument Setup
//!
//! Resolve text-heavy external symbols into cheap [`InstrumentId`] values before
//! the pretrade hot path:
//!
//! ```
//! use risk_core::{
//!     CurrencyId, EquitySpec, Instrument, InstrumentCatalog, InstrumentId,
//!     SymbolKey, SymbolRegistry,
//! };
//!
//! let symbol = SymbolKey {
//!     venue: "XNYS".to_owned(),
//!     symbol: "IBM".to_owned(),
//! };
//! let instrument = Instrument::Equity(EquitySpec {
//!     instrument_id: InstrumentId(1),
//!     settlement_currency: CurrencyId(840),
//! });
//!
//! let mut registry = SymbolRegistry::new();
//! registry.register(symbol.clone(), InstrumentId(1)).unwrap();
//!
//! let mut catalog = InstrumentCatalog::new();
//! catalog.insert(instrument).unwrap();
//!
//! let resolved = registry.resolve(&symbol).unwrap();
//! assert_eq!(catalog.get(resolved), Some(instrument));
//! ```
//!
//! # Trusted Market Data
//!
//! [`MarketSnapshot`] centralizes freshness and data-quality checks:
//!
//! ```
//! use risk_core::{
//!     DataQuality, InstrumentId, MarketPrice, MarketSnapshot, Notional, Price,
//!     Timestamp,
//! };
//!
//! let mut market = MarketSnapshot::new(10, 10, 10);
//! market.insert_price(
//!     InstrumentId(1),
//!     MarketPrice::clean(Price::new(100), Timestamp(5)),
//! );
//! market.set_aggregate_notional(
//!     Notional::new(0),
//!     Timestamp(5),
//!     DataQuality::clean(),
//! );
//!
//! assert_eq!(
//!     market.trusted_price(InstrumentId(1), Timestamp(10)).unwrap(),
//!     Price::new(100)
//! );
//! ```

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
