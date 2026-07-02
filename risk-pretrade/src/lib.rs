//! Synchronous pretrade risk gate.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod checks;
pub mod gate;
pub mod limit_source;

pub use gate::{EvaluateRequest, LimitTable, PretradeGate};
pub use limit_source::{
    FileLimitSource, LimitSource, ParseLimitTableError, ParseLimitTableErrorKind,
    StaticLimitSource, parse_limit_table,
};
