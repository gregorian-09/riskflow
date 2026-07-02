//! Fat-finger price band checks.

use risk_core::{
    IndeterminateReason, InstrumentId, MarketSnapshot, Price, RejectReason, RiskVerdict, Timestamp,
};

use crate::gate::LimitTable;

const BPS_DENOMINATOR: i128 = 10_000;

/// Checks submitted order price against the trusted market price.
#[must_use]
pub fn check(
    limits: &LimitTable,
    instrument_id: InstrumentId,
    order_price: Price,
    market: &MarketSnapshot,
    now: Timestamp,
) -> RiskVerdict {
    let Some(band_bps) = limits.fat_finger_band_bps(instrument_id) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::MissingLimit);
    };

    let reference_price = match market.trusted_price(instrument_id, now) {
        Ok(price) => price,
        Err(reason) => return RiskVerdict::Indeterminate(reason),
    };

    let Some(reference_abs) = i128::from(reference_price.raw()).checked_abs() else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    if reference_abs == 0 {
        return RiskVerdict::Indeterminate(IndeterminateReason::BadDataQuality);
    }

    let Some(delta) = i128::from(order_price.raw())
        .checked_sub(i128::from(reference_price.raw()))
        .and_then(i128::checked_abs)
    else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    let Some(scaled_delta) = delta.checked_mul(BPS_DENOMINATOR) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    let Some(max_delta) = reference_abs.checked_mul(i128::from(band_bps)) else {
        return RiskVerdict::Indeterminate(IndeterminateReason::ArithmeticOverflow);
    };

    if scaled_delta > max_delta {
        RiskVerdict::Reject(RejectReason::FatFinger)
    } else {
        RiskVerdict::Pass
    }
}
