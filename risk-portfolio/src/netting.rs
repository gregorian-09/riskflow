//! Cross-currency netting placeholders.

use risk_core::{IndeterminateReason, MarketSnapshot, Timestamp};

/// Returns whether the aggregate exposure snapshot is fresh enough for netting.
pub fn aggregate_snapshot_ready(
    market: &MarketSnapshot,
    now: Timestamp,
) -> Result<(), IndeterminateReason> {
    if market.aggregate_is_fresh(now) {
        Ok(())
    } else {
        Err(IndeterminateReason::StaleAggregateSnapshot)
    }
}
