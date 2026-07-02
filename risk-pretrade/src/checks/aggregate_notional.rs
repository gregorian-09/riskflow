//! Aggregate notional limit checks.

use risk_core::{
    IndeterminateReason, MarketSnapshot, Notional, RejectReason, RiskVerdict, Timestamp,
};

use crate::gate::LimitTable;

/// Checks post-order aggregate notional against the configured limit.
#[must_use]
pub fn check(
    limits: &LimitTable,
    market: &MarketSnapshot,
    now: Timestamp,
    order_notional: Notional,
) -> RiskVerdict {
    let Some(limit) = limits.aggregate_notional_limit() else {
        return RiskVerdict::Indeterminate(IndeterminateReason::MissingLimit);
    };

    let aggregate_notional = match market.trusted_aggregate_notional(now) {
        Ok(notional) => notional,
        Err(reason) => return RiskVerdict::Indeterminate(reason),
    };

    let Some(post_order_notional) = aggregate_notional.checked_add(order_notional) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    if post_order_notional > limit {
        RiskVerdict::Reject(RejectReason::AggregateNotionalLimit)
    } else {
        RiskVerdict::Pass
    }
}
