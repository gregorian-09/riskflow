//! Shared types for the Risk workspace.
//!
//! `risk-core` owns the fixed-point identifiers, instruments, positions,
//! market snapshots, and verdict types consumed by latency-sensitive and
//! offline risk crates.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod currency;
pub mod instrument;
pub mod market;
pub mod position;
pub mod symbol;
pub mod types;
pub mod verdict;

pub use currency::{CurrencyId, CurrencyPair};
pub use instrument::{
    AssetClass, CryptoSpotSpec, CurrencySet, EquitySpec, FutureSpec, FxSpec, Instrument,
    OptionSpec, PerpSpec,
};
pub use market::{DataQuality, MarketPrice, MarketSnapshot, RiskDataQualityFlags};
pub use position::{Funding, MarginState, Position};
pub use symbol::{RegisterSymbolError, SymbolKey, SymbolRegistry};
pub use types::{InstrumentId, Notional, Price, Qty, Timestamp};
pub use verdict::{IndeterminateReason, RejectReason, RiskVerdict, RiskWeight};
