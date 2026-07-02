//! Market prices, FX rates, and data-quality state.

use std::collections::HashMap;

use crate::{
    currency::CurrencyPair,
    types::{InstrumentId, Price, Timestamp},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DataQuality {
    /// Raw upstream quality flags.
    ///
    /// This is reserved for `of_core::DataQualityFlags` once the external
    /// dependency is wired in. Keeping it as raw bits for the first slice avoids
    /// inventing a competing upstream flag taxonomy.
    pub upstream_bits: u32,
    /// Risk-specific quality flags not represented upstream.
    pub risk_flags: RiskDataQualityFlags,
}

impl DataQuality {
    /// Returns clean quality with no upstream or risk-local flags.
    #[must_use]
    pub const fn clean() -> Self {
        Self {
            upstream_bits: 0,
            risk_flags: RiskDataQualityFlags::EMPTY,
        }
    }

    /// Returns whether no quality flags are set.
    #[must_use]
    pub const fn is_clean(self) -> bool {
        self.upstream_bits == 0 && !self.risk_flags.any()
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

    /// Marks the aggregate exposure snapshot observation time.
    pub const fn set_aggregate_observed_at(&mut self, observed_at: Timestamp) {
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

    /// Returns whether the aggregate exposure snapshot is fresh enough.
    #[must_use]
    pub fn aggregate_is_fresh(&self, now: Timestamp) -> bool {
        self.aggregate_observed_at
            .is_some_and(|observed_at| observed_at.age_at(now) <= self.max_aggregate_age_nanos)
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
