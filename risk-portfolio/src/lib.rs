//! Offline portfolio analytics.
//!
//! The v1 crate shape is reserved here so analytics modules can grow without
//! entering the pretrade hot path.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod netting;
pub mod performance;
pub mod var;
