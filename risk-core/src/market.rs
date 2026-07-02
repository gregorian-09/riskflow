//! Market prices, FX rates, and data-quality state.

use std::collections::HashMap;

use of_core::DataQualityFlags;

use crate::{
    currency::CurrencyPair,
    types::{InstrumentId, Notional, Price, Timestamp},
    verdict::IndeterminateReason,
};

/// Risk-specific market data quality flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RiskDataQualityFlags(u32);

impl RiskDataQualityFlags {
    /// No risk-specific quality flags are set.
    pub const EMPTY: Self = Self(0);
    /// FX conversion data is stale.
    pub const STALE_FX_RATE: Self = Self(1 << 0);
    /// Aggregate exposure snapshot is stale.
    pub const STALE_AGGREGATE: Self = Self(1 << 1);
    /// Multiple sources disagree outside configured tolerance.
    pub const SOURCE_DISAGREEMENT: Self = Self(1 << 2);

    /// Returns `true` if any flags are set.
    #[must_use]
    pub const fn any(self) -> bool {
        self.0 != 0
    }

    /// Returns the raw bit representation.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns the union of two flag sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Combined upstream and risk-local market data quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataQuality {
    /// Upstream Orderflow data-quality flags.
    pub upstream_flags: DataQualityFlags,
    /// Risk-specific quality flags not represented upstream.
    pub risk_flags: RiskDataQualityFlags,
}

impl Default for DataQuality {
    fn default() -> Self {
        Self::clean()
    }
}

impl DataQuality {
    /// Returns clean quality with no upstream or risk-local flags.
    #[must_use]
    pub const fn clean() -> Self {
        Self {
            upstream_flags: DataQualityFlags::NONE,
            risk_flags: RiskDataQualityFlags::EMPTY,
        }
    }

    /// Creates data quality from upstream Orderflow flags.
    #[must_use]
    pub const fn from_upstream(upstream_flags: DataQualityFlags) -> Self {
        Self {
            upstream_flags,
            risk_flags: RiskDataQualityFlags::EMPTY,
        }
    }

    /// Returns whether no quality flags are set.
    #[must_use]
    pub fn is_clean(self) -> bool {
        self.upstream_flags.bits() == 0 && !self.risk_flags.any()
    }
}

/// Timestamped market price with quality metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketPrice {
    /// Fixed-point price.
    pub price: Price,
    /// Observation timestamp.
    pub observed_at: Timestamp,
    /// Data quality flags.
    pub quality: DataQuality,
}

impl MarketPrice {
    /// Creates a clean market price.
    #[must_use]
    pub const fn clean(price: Price, observed_at: Timestamp) -> Self {
        Self {
            price,
            observed_at,
            quality: DataQuality::clean(),
        }
    }
}

/// Immutable market snapshot consumed by risk checks.
#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    prices: HashMap<InstrumentId, MarketPrice>,
    fx_rates: HashMap<CurrencyPair, MarketPrice>,
    max_price_age_nanos: u64,
    max_fx_age_nanos: u64,
    aggregate_notional: Option<Notional>,
    aggregate_quality: DataQuality,
    aggregate_observed_at: Option<Timestamp>,
    max_aggregate_age_nanos: u64,
}

impl MarketSnapshot {
    /// Creates an empty market snapshot with explicit freshness tolerances.
    #[must_use]
    pub fn new(
        max_price_age_nanos: u64,
        max_fx_age_nanos: u64,
        max_aggregate_age_nanos: u64,
    ) -> Self {
        Self {
            prices: HashMap::new(),
            fx_rates: HashMap::new(),
            max_price_age_nanos,
            max_fx_age_nanos,
            aggregate_notional: None,
            aggregate_quality: DataQuality::clean(),
            aggregate_observed_at: None,
            max_aggregate_age_nanos,
        }
    }

    /// Inserts or replaces an instrument price.
    pub fn insert_price(&mut self, instrument_id: InstrumentId, price: MarketPrice) {
        self.prices.insert(instrument_id, price);
    }

    /// Inserts or replaces an FX rate.
    pub fn insert_fx_rate(&mut self, pair: CurrencyPair, price: MarketPrice) {
        self.fx_rates.insert(pair, price);
    }

    /// Sets the aggregate base-currency notional snapshot.
    pub const fn set_aggregate_notional(
        &mut self,
        notional: Notional,
        observed_at: Timestamp,
        quality: DataQuality,
    ) {
        self.aggregate_notional = Some(notional);
        self.aggregate_quality = quality;
        self.aggregate_observed_at = Some(observed_at);
    }

    /// Returns a trusted instrument price or an indeterminate reason.
    pub fn trusted_price(
        &self,
        instrument_id: InstrumentId,
        now: Timestamp,
    ) -> Result<Price, IndeterminateReason> {
        let price = self
            .prices
            .get(&instrument_id)
            .ok_or(IndeterminateReason::MissingPrice)?;

        trusted_market_price(
            *price,
            now,
            self.max_price_age_nanos,
            IndeterminateReason::StalePrice,
        )
    }

    /// Returns a trusted FX rate or an indeterminate reason.
    pub fn trusted_fx_rate(
        &self,
        pair: CurrencyPair,
        now: Timestamp,
    ) -> Result<Price, IndeterminateReason> {
        let price = self
            .fx_rates
            .get(&pair)
            .ok_or(IndeterminateReason::MissingFxRate)?;

        trusted_market_price(
            *price,
            now,
            self.max_fx_age_nanos,
            IndeterminateReason::StaleFxRate,
        )
    }

    /// Converts notional through a trusted FX rate.
    ///
    /// The raw fixed-point scale is intentionally owned by the caller's
    /// instrument and currency configuration. This method validates freshness,
    /// data quality, and overflow before returning the raw product.
    pub fn convert_notional(
        &self,
        notional: Notional,
        pair: CurrencyPair,
        now: Timestamp,
    ) -> Result<Notional, IndeterminateReason> {
        let fx_rate = self.trusted_fx_rate(pair, now)?;

        notional
            .checked_mul_price(fx_rate)
            .ok_or(IndeterminateReason::ArithmeticOverflow)
    }

    /// Verifies that two instrument price sources agree within a BPS tolerance.
    ///
    /// Both prices must first pass the snapshot's normal price trust checks.
    /// Tolerance is measured against the larger absolute source price so the
    /// check is symmetric.
    pub fn validate_source_agreement(
        &self,
        primary: InstrumentId,
        secondary: InstrumentId,
        max_deviation_bps: u32,
        now: Timestamp,
    ) -> Result<(), IndeterminateReason> {
        let primary = self.trusted_price(primary, now)?;
        let secondary = self.trusted_price(secondary, now)?;

        prices_agree(primary, secondary, max_deviation_bps)
    }

    /// Returns whether the aggregate exposure snapshot is fresh enough.
    #[must_use]
    pub fn aggregate_is_fresh(&self, now: Timestamp) -> bool {
        self.aggregate_observed_at
            .is_some_and(|observed_at| observed_at.age_at(now) <= self.max_aggregate_age_nanos)
    }

    /// Returns a trusted aggregate base-currency notional snapshot.
    pub fn trusted_aggregate_notional(
        &self,
        now: Timestamp,
    ) -> Result<Notional, IndeterminateReason> {
        let notional = self
            .aggregate_notional
            .ok_or(IndeterminateReason::MissingAggregateSnapshot)?;

        if !self.aggregate_quality.is_clean() {
            return Err(IndeterminateReason::BadDataQuality);
        }

        if !self.aggregate_is_fresh(now) {
            return Err(IndeterminateReason::StaleAggregateSnapshot);
        }

        Ok(notional)
    }
}

fn trusted_market_price(
    price: MarketPrice,
    now: Timestamp,
    max_age_nanos: u64,
    stale_reason: IndeterminateReason,
) -> Result<Price, IndeterminateReason> {
    if !price.quality.is_clean() {
        return Err(IndeterminateReason::BadDataQuality);
    }

    if price.observed_at.age_at(now) > max_age_nanos {
        return Err(stale_reason);
    }

    Ok(price.price)
}

fn prices_agree(
    primary: Price,
    secondary: Price,
    max_deviation_bps: u32,
) -> Result<(), IndeterminateReason> {
    let primary = i128::from(primary.raw());
    let secondary = i128::from(secondary.raw());
    let denominator = primary.abs().max(secondary.abs());

    if denominator == 0 {
        return Ok(());
    }

    let deviation = (primary - secondary).abs();
    let deviation_bps = deviation
        .checked_mul(10_000)
        .ok_or(IndeterminateReason::ArithmeticOverflow)?
        / denominator;

    if deviation_bps <= i128::from(max_deviation_bps) {
        Ok(())
    } else {
        Err(IndeterminateReason::SourceDisagreement)
    }
}

#[cfg(test)]
mod tests {
    use of_core::DataQualityFlags;

    use crate::{
        currency::{CurrencyId, CurrencyPair},
        market::{DataQuality, MarketPrice, MarketSnapshot},
        types::{InstrumentId, Notional, Price, Timestamp},
        verdict::IndeterminateReason,
    };

    #[test]
    fn upstream_quality_flags_fail_closed() {
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice {
                price: Price::new(100),
                observed_at: Timestamp(5),
                quality: DataQuality::from_upstream(DataQualityFlags::STALE_FEED),
            },
        );

        assert_eq!(
            market.trusted_price(InstrumentId(1), Timestamp(10)),
            Err(IndeterminateReason::BadDataQuality)
        );
    }

    #[test]
    fn convert_notional_uses_trusted_fx_rate() {
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_fx_rate(
            CurrencyPair::new(CurrencyId(1), CurrencyId(2)),
            MarketPrice::clean(Price::new(2), Timestamp(5)),
        );

        assert_eq!(
            market.convert_notional(
                Notional::new(100),
                CurrencyPair::new(CurrencyId(1), CurrencyId(2)),
                Timestamp(10)
            ),
            Ok(Notional::new(200))
        );
    }

    #[test]
    fn convert_notional_fails_closed_for_stale_fx() {
        let mut market = MarketSnapshot::new(10, 2, 10);
        market.insert_fx_rate(
            CurrencyPair::new(CurrencyId(1), CurrencyId(2)),
            MarketPrice::clean(Price::new(2), Timestamp(5)),
        );

        assert_eq!(
            market.convert_notional(
                Notional::new(100),
                CurrencyPair::new(CurrencyId(1), CurrencyId(2)),
                Timestamp(10)
            ),
            Err(IndeterminateReason::StaleFxRate)
        );
    }

    #[test]
    fn source_agreement_accepts_prices_inside_tolerance() {
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(10_000), Timestamp(5)),
        );
        market.insert_price(
            InstrumentId(2),
            MarketPrice::clean(Price::new(10_020), Timestamp(5)),
        );

        assert_eq!(
            market.validate_source_agreement(InstrumentId(1), InstrumentId(2), 25, Timestamp(10)),
            Ok(())
        );
    }

    #[test]
    fn source_agreement_fails_closed_outside_tolerance() {
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(10_000), Timestamp(5)),
        );
        market.insert_price(
            InstrumentId(2),
            MarketPrice::clean(Price::new(10_500), Timestamp(5)),
        );

        assert_eq!(
            market.validate_source_agreement(InstrumentId(1), InstrumentId(2), 25, Timestamp(10)),
            Err(IndeterminateReason::SourceDisagreement)
        );
    }
}
