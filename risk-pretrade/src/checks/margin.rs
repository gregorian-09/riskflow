//! Futures and perpetual swap margin checks.

use risk_core::{
    AssetClass, IndeterminateReason, Instrument, InstrumentId, Notional, Qty, RejectReason,
    RiskVerdict,
};

use crate::gate::LimitTable;

/// Checks initial margin for futures and perpetual swaps.
#[must_use]
pub fn check(
    limits: &LimitTable,
    instrument: Instrument,
    current_position: Qty,
    order_qty: Qty,
    available_margin: Notional,
) -> RiskVerdict {
    if !matches!(
        instrument.asset_class(),
        AssetClass::Future | AssetClass::PerpetualSwap
    ) {
        return RiskVerdict::Pass;
    }

    check_margined_instrument(
        limits,
        instrument.id(),
        current_position,
        order_qty,
        available_margin,
    )
}

fn check_margined_instrument(
    limits: &LimitTable,
    instrument_id: InstrumentId,
    current_position: Qty,
    order_qty: Qty,
    available_margin: Notional,
) -> RiskVerdict {
    let Some(initial_margin_per_unit) = limits.initial_margin_per_unit(instrument_id) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::MissingLimit);
    };

    let Some(post_order_position) = current_position.checked_add(order_qty) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    let Some(required_margin) = initial_margin_per_unit.checked_mul_abs_qty(post_order_position)
    else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    if required_margin > available_margin {
        RiskVerdict::Reject(RejectReason::Margin)
    } else {
        RiskVerdict::Pass
    }
}
