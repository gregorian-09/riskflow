//! Notional limit checks.

use risk_core::{IndeterminateReason, InstrumentId, Notional, RejectReason, RiskVerdict};

use crate::gate::LimitTable;

/// Checks per-order notional against configured limits.
#[must_use]
pub fn check_per_order(
    limits: &LimitTable,
    instrument_id: InstrumentId,
    order_notional: Notional,
) -> RiskVerdict {
    let Some(limit) = limits.per_order_notional(instrument_id) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::MissingLimit);
    };

    if order_notional > limit {
        RiskVerdict::Reject(RejectReason::OrderNotionalLimit)
    } else {
        RiskVerdict::Pass
    }
}
