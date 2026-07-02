//! Risk result and uncertainty types.

use crate::types::Notional;

/// Risk weight consumed by pretrade and portfolio checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskWeight {
    /// Linear notional exposure.
    Linear(Notional),
    /// Exposure cannot be trusted or computed.
    Indeterminate(IndeterminateReason),
}

/// Final result of a risk check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskVerdict {
    /// The order passed all evaluated checks.
    Pass,
    /// The order failed a deterministic risk limit.
    Reject(RejectReason),
    /// The gate could not produce a trustworthy answer and must fail closed.
    Indeterminate(IndeterminateReason),
}

impl RiskVerdict {
    /// Returns `true` only for `Pass`.
    #[must_use]
    pub const fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// Deterministic reject reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Per-order notional limit would be exceeded.
    OrderNotionalLimit,
    /// Aggregate notional limit would be exceeded.
    AggregateNotionalLimit,
    /// Position limit would be exceeded.
    PositionLimit,
    /// Price band check failed.
    FatFinger,
    /// Margin requirement check failed.
    Margin,
    /// Instrument is unknown to the gate.
    UnknownInstrument,
}

/// Reasons a check could not produce a trustworthy decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndeterminateReason {
    /// No market price exists for the instrument.
    MissingPrice,
    /// The market price exists but is stale.
    StalePrice,
    /// Upstream or local data-quality flags mark the input unsafe.
    BadDataQuality,
    /// A required FX rate is missing.
    MissingFxRate,
    /// A required FX rate is stale.
    StaleFxRate,
    /// Multiple market data sources disagree outside the configured tolerance.
    SourceDisagreement,
    /// The aggregate exposure snapshot is stale.
    StaleAggregateSnapshot,
    /// Fixed-point arithmetic overflowed.
    ArithmeticOverflow,
    /// Options are deliberately unpriced in v1.
    UnsupportedOption,
    /// Symbol resolution was attempted for an unknown symbol.
    UnknownSymbol,
    /// Limit data is missing or stale.
    MissingLimit,
    /// Aggregate exposure snapshot is unavailable.
    MissingAggregateSnapshot,
}
