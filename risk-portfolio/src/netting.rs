//! Cross-currency netting helpers.

use risk_core::{
    CurrencyPair, IndeterminateReason, InstrumentId, MarketSnapshot, Notional, Timestamp,
};

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

/// Returns the trusted aggregate notional snapshot for reporting.
pub fn aggregate_notional_snapshot(
    market: &MarketSnapshot,
    now: Timestamp,
) -> Result<Notional, IndeterminateReason> {
    market.trusted_aggregate_notional(now)
}

/// Converts a notional into another currency using a trusted FX rate.
pub fn convert_notional_to_currency(
    market: &MarketSnapshot,
    notional: Notional,
    pair: CurrencyPair,
    now: Timestamp,
) -> Result<Notional, IndeterminateReason> {
    market.convert_notional(notional, pair, now)
}

/// Verifies that two price sources are close enough to net positions safely.
pub fn price_sources_agree(
    market: &MarketSnapshot,
    primary: InstrumentId,
    secondary: InstrumentId,
    max_deviation_bps: u32,
    now: Timestamp,
) -> Result<(), IndeterminateReason> {
    market.validate_source_agreement(primary, secondary, max_deviation_bps, now)
}

#[cfg(test)]
mod tests {
    use risk_core::{
        CurrencyId, CurrencyPair, InstrumentId, MarketPrice, MarketSnapshot, Notional, Price,
        Timestamp,
    };

    use super::{convert_notional_to_currency, price_sources_agree};

    #[test]
    fn converts_notional_with_snapshot_fx_policy() {
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_fx_rate(
            CurrencyPair::new(CurrencyId(1), CurrencyId(2)),
            MarketPrice::clean(Price::new(3), Timestamp(8)),
        );

        assert_eq!(
            convert_notional_to_currency(
                &market,
                Notional::new(40),
                CurrencyPair::new(CurrencyId(1), CurrencyId(2)),
                Timestamp(10)
            ),
            Ok(Notional::new(120))
        );
    }

    #[test]
    fn validates_price_sources_through_snapshot_policy() {
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(1_000), Timestamp(8)),
        );
        market.insert_price(
            InstrumentId(2),
            MarketPrice::clean(Price::new(1_001), Timestamp(8)),
        );

        assert_eq!(
            price_sources_agree(&market, InstrumentId(1), InstrumentId(2), 20, Timestamp(10)),
            Ok(())
        );
    }
}
