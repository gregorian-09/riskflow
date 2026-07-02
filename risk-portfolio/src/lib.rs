//! Offline portfolio analytics.
//!
//! The v1 crate shape is reserved here so analytics modules can grow without
//! entering the pretrade hot path.
//!
//! Portfolio analytics are batch-oriented: allocation and `f64` statistics are
//! acceptable here, while settlement and pretrade limit math remain fixed-point
//! in `risk-core` and `risk-pretrade`.
//!
//! # Scope
//!
//! v1 includes historical, parametric, and seeded Monte Carlo `VaR`, performance
//! ratios, drawdown metrics, and trusted cross-currency netting helpers.
//! Options Greeks stay out of this crate until the isolated options layer
//! exists.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod netting;
pub mod performance;
pub mod var;
