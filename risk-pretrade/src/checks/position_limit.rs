//! Position limit checks.

use risk_core::{IndeterminateReason, InstrumentId, Qty, RejectReason, RiskVerdict};

use crate::gate::LimitTable;

/// Checks absolute post-order position against configured limits.
#[must_use]
pub fn check(
    limits: &LimitTable,
    instrument_id: InstrumentId,
    current_position: Qty,
    order_qty: Qty,
) -> RiskVerdict {
    let Some(limit) = limits.max_abs_position(instrument_id) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::MissingLimit);
    };

    let Some(post_order_position) = current_position.checked_add(order_qty) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    let Some(abs_position) = post_order_position.checked_abs() else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    let Some(abs_limit) = limit.checked_abs() else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    if abs_position > abs_limit {
        RiskVerdict::Reject(RejectReason::PositionLimit)
    } else {
        RiskVerdict::Pass
    }
}
