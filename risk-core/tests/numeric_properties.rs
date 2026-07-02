//! Property tests for fixed-point primitive behavior.

use proptest::prelude::*;
use risk_core::{Notional, Price, Qty, Timestamp};

proptest! {
    #[test]
    fn checked_add_matches_i64_checked_add(lhs in any::<i64>(), rhs in any::<i64>()) {
        let expected = lhs.checked_add(rhs).map(Notional::new);

        prop_assert_eq!(Notional::new(lhs).checked_add(Notional::new(rhs)), expected);
    }

    #[test]
    fn checked_mul_price_matches_i128_reference(notional in any::<i64>(), rate in any::<i64>()) {
        let expected = i128::from(notional)
            .checked_mul(i128::from(rate))
            .and_then(|raw| i64::try_from(raw).ok())
            .map(Notional::new);

        prop_assert_eq!(
            Notional::new(notional).checked_mul_price(Price::new(rate)),
            expected
        );
    }

    #[test]
    fn checked_mul_abs_qty_matches_i128_reference(unit in any::<i64>(), qty in any::<i64>()) {
        let expected = i128::from(unit)
            .checked_mul(i128::from(qty))
            .and_then(i128::checked_abs)
            .and_then(|raw| i64::try_from(raw).ok())
            .map(Notional::new);

        prop_assert_eq!(
            Notional::new(unit).checked_mul_abs_qty(Qty::new(qty)),
            expected
        );
    }

    #[test]
    fn checked_linear_matches_i128_reference(
        price in any::<i64>(),
        qty in any::<i64>(),
        multiplier in any::<i64>(),
    ) {
        let expected = i128::from(price)
            .checked_mul(i128::from(qty))
            .and_then(|raw| raw.checked_mul(i128::from(multiplier)))
            .and_then(i128::checked_abs)
            .and_then(|raw| i64::try_from(raw).ok())
            .map(Notional::new);

        prop_assert_eq!(
            Notional::checked_linear(Price::new(price), Qty::new(qty), multiplier),
            expected
        );
    }

    #[test]
    fn timestamp_age_saturates(observed_at in any::<u64>(), now in any::<u64>()) {
        prop_assert_eq!(
            Timestamp(observed_at).age_at(Timestamp(now)),
            now.saturating_sub(observed_at)
        );
    }
}
