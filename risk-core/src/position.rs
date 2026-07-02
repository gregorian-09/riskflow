//! Dynamic position state.

use crate::types::{InstrumentId, Qty, Timestamp};

/// Fixed-point accrued funding amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Funding(pub i64);

/// Margin state for margined instruments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarginState {
    /// Initial margin requirement.
    pub initial: i64,
    /// Maintenance margin requirement.
    pub maintenance: i64,
}

/// Dynamic position state, separate from static instrument reference data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    /// Cash equity position.
    Equity {
        /// Instrument identity.
        instrument_id: InstrumentId,
        /// Signed position quantity.
        qty: Qty,
    },
    /// Spot FX position.
    SpotFx {
        /// Instrument identity.
        instrument_id: InstrumentId,
        /// Signed position quantity.
        qty: Qty,
    },
    /// Spot crypto position.
    SpotCrypto {
        /// Instrument identity.
        instrument_id: InstrumentId,
        /// Signed position quantity.
        qty: Qty,
    },
    /// Futures position.
    Future {
        /// Instrument identity.
        instrument_id: InstrumentId,
        /// Signed position quantity.
        qty: Qty,
        /// Margin state.
        margin: MarginState,
        /// Contract expiry.
        expiry: Timestamp,
    },
    /// Perpetual swap position.
    Perp {
        /// Instrument identity.
        instrument_id: InstrumentId,
        /// Signed position quantity.
        qty: Qty,
        /// Margin state.
        margin: MarginState,
        /// Accrued funding.
        accrued_funding: Funding,
    },
    /// Option position placeholder.
    Option {
        /// Instrument identity.
        instrument_id: InstrumentId,
        /// Signed position quantity.
        qty: Qty,
        /// Optional externally supplied delta, unused by v1 pretrade.
        delta: Option<f64>,
        /// Contract expiry.
        expiry: Timestamp,
    },
}

impl Position {
    /// Returns the instrument identity for this position.
    #[must_use]
    pub const fn instrument_id(self) -> InstrumentId {
        match self {
            Self::Equity { instrument_id, .. }
            | Self::SpotFx { instrument_id, .. }
            | Self::SpotCrypto { instrument_id, .. }
            | Self::Future { instrument_id, .. }
            | Self::Perp { instrument_id, .. }
            | Self::Option { instrument_id, .. } => instrument_id,
        }
    }

    /// Returns the signed position quantity.
    #[must_use]
    pub const fn qty(self) -> Qty {
        match self {
            Self::Equity { qty, .. }
            | Self::SpotFx { qty, .. }
            | Self::SpotCrypto { qty, .. }
            | Self::Future { qty, .. }
            | Self::Perp { qty, .. }
            | Self::Option { qty, .. } => qty,
        }
    }
}
