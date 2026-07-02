//! Instrument taxonomy and risk exposure behavior.

use std::{collections::HashMap, error::Error, fmt};

use crate::{
    currency::CurrencyId,
    market::MarketSnapshot,
    types::{InstrumentId, Notional, Qty, Timestamp},
    verdict::{IndeterminateReason, RiskWeight},
};

/// Supported v1 asset classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetClass {
    /// Cash equity.
    Equity,
    /// Spot foreign exchange.
    SpotFx,
    /// Spot crypto asset.
    SpotCrypto,
    /// Exchange-traded future.
    Future,
    /// Perpetual swap.
    PerpetualSwap,
    /// Option placeholder; unpriced in v1.
    Option,
}

/// Static equity contract specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquitySpec {
    /// Instrument identity.
    pub instrument_id: InstrumentId,
    /// Settlement currency.
    pub settlement_currency: CurrencyId,
}

/// Static spot FX contract specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FxSpec {
    /// Instrument identity.
    pub instrument_id: InstrumentId,
    /// Base currency.
    pub base_currency: CurrencyId,
    /// Quote/settlement currency.
    pub quote_currency: CurrencyId,
}

/// Static spot crypto contract specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CryptoSpotSpec {
    /// Instrument identity.
    pub instrument_id: InstrumentId,
    /// Base asset identifier.
    pub base_currency: CurrencyId,
    /// Quote/settlement currency.
    pub quote_currency: CurrencyId,
}

/// Static futures contract specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FutureSpec {
    /// Instrument identity.
    pub instrument_id: InstrumentId,
    /// Settlement currency.
    pub settlement_currency: CurrencyId,
    /// Fixed-point contract multiplier.
    pub multiplier: i64,
}

/// Static perpetual swap contract specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerpSpec {
    /// Instrument identity.
    pub instrument_id: InstrumentId,
    /// Settlement currency.
    pub settlement_currency: CurrencyId,
    /// Fixed-point contract multiplier.
    pub multiplier: i64,
}

/// Static option contract placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionSpec {
    /// Instrument identity.
    pub instrument_id: InstrumentId,
    /// Underlying instrument identity.
    pub underlying_id: InstrumentId,
    /// Settlement currency.
    pub settlement_currency: CurrencyId,
    /// Expiry timestamp.
    pub expiry: Timestamp,
}

/// Closed instrument enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    /// Cash equity.
    Equity(EquitySpec),
    /// Spot foreign exchange.
    SpotFx(FxSpec),
    /// Spot crypto asset.
    SpotCrypto(CryptoSpotSpec),
    /// Exchange-traded future.
    Future(FutureSpec),
    /// Perpetual swap.
    Perp(PerpSpec),
    /// Option placeholder; unpriced in v1.
    Option(OptionSpec),
}

/// Common exposure interface consumed across risk crates.
///
/// This trait is the stable boundary a future options implementation can
/// satisfy without making `risk-pretrade` or `risk-portfolio` depend on an
/// options crate directly.
pub trait RiskExposure {
    /// Computes the risk weight for the supplied quantity and market snapshot.
    fn risk_weight(&self, qty: Qty, market: &MarketSnapshot, now: Timestamp) -> RiskWeight;

    /// Returns currencies touched by the exposure.
    fn currencies(&self) -> CurrencySet;

    /// Returns the settlement currency for the exposure.
    fn settlement_currency(&self) -> CurrencyId;
}

/// Startup-loaded instrument reference data.
#[derive(Debug, Clone, Default)]
pub struct InstrumentCatalog {
    by_id: HashMap<InstrumentId, Instrument>,
}

impl InstrumentCatalog {
    /// Creates an empty instrument catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts an instrument, rejecting duplicate instrument ids.
    pub fn insert(&mut self, instrument: Instrument) -> Result<(), InstrumentCatalogError> {
        let instrument_id = instrument.id();
        if self.by_id.contains_key(&instrument_id) {
            return Err(InstrumentCatalogError::DuplicateInstrumentId(instrument_id));
        }

        self.by_id.insert(instrument_id, instrument);
        Ok(())
    }

    /// Returns an instrument by id.
    #[must_use]
    pub fn get(&self, instrument_id: InstrumentId) -> Option<Instrument> {
        self.by_id.get(&instrument_id).copied()
    }

    /// Returns the number of instruments in the catalog.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns whether the catalog is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// Instrument catalog construction error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentCatalogError {
    /// Instrument id was already inserted.
    DuplicateInstrumentId(InstrumentId),
}

impl fmt::Display for InstrumentCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateInstrumentId(instrument_id) => {
                write!(f, "duplicate instrument id {}", instrument_id.raw())
            }
        }
    }
}

impl Error for InstrumentCatalogError {}

impl Instrument {
    /// Returns the instrument identity.
    #[must_use]
    pub const fn id(self) -> InstrumentId {
        match self {
            Self::Equity(spec) => spec.instrument_id,
            Self::SpotFx(spec) => spec.instrument_id,
            Self::SpotCrypto(spec) => spec.instrument_id,
            Self::Future(spec) => spec.instrument_id,
            Self::Perp(spec) => spec.instrument_id,
            Self::Option(spec) => spec.instrument_id,
        }
    }

    /// Returns the asset class.
    #[must_use]
    pub const fn asset_class(self) -> AssetClass {
        match self {
            Self::Equity(_) => AssetClass::Equity,
            Self::SpotFx(_) => AssetClass::SpotFx,
            Self::SpotCrypto(_) => AssetClass::SpotCrypto,
            Self::Future(_) => AssetClass::Future,
            Self::Perp(_) => AssetClass::PerpetualSwap,
            Self::Option(_) => AssetClass::Option,
        }
    }

    /// Computes the risk weight for this instrument.
    #[must_use]
    pub fn risk_weight(self, qty: Qty, market: &MarketSnapshot, now: Timestamp) -> RiskWeight {
        match self {
            Self::Equity(spec) => linear_weight(spec.instrument_id, qty, 1, market, now),
            Self::SpotFx(spec) => linear_weight(spec.instrument_id, qty, 1, market, now),
            Self::SpotCrypto(spec) => linear_weight(spec.instrument_id, qty, 1, market, now),
            Self::Future(spec) => {
                linear_weight(spec.instrument_id, qty, spec.multiplier, market, now)
            }
            Self::Perp(spec) => {
                linear_weight(spec.instrument_id, qty, spec.multiplier, market, now)
            }
            Self::Option(_) => RiskWeight::Indeterminate(IndeterminateReason::UnsupportedOption),
        }
    }

    /// Returns currencies touched by the instrument.
    #[must_use]
    pub const fn currencies(self) -> CurrencySet {
        match self {
            Self::Equity(spec) => CurrencySet::one(spec.settlement_currency),
            Self::SpotFx(spec) => CurrencySet::two(spec.base_currency, spec.quote_currency),
            Self::SpotCrypto(spec) => CurrencySet::two(spec.base_currency, spec.quote_currency),
            Self::Future(spec) => CurrencySet::one(spec.settlement_currency),
            Self::Perp(spec) => CurrencySet::one(spec.settlement_currency),
            Self::Option(spec) => CurrencySet::one(spec.settlement_currency),
        }
    }

    /// Returns settlement currency.
    #[must_use]
    pub const fn settlement_currency(self) -> CurrencyId {
        match self {
            Self::Equity(spec) => spec.settlement_currency,
            Self::SpotFx(spec) => spec.quote_currency,
            Self::SpotCrypto(spec) => spec.quote_currency,
            Self::Future(spec) => spec.settlement_currency,
            Self::Perp(spec) => spec.settlement_currency,
            Self::Option(spec) => spec.settlement_currency,
        }
    }
}

impl RiskExposure for Instrument {
    fn risk_weight(&self, qty: Qty, market: &MarketSnapshot, now: Timestamp) -> RiskWeight {
        (*self).risk_weight(qty, market, now)
    }

    fn currencies(&self) -> CurrencySet {
        (*self).currencies()
    }

    fn settlement_currency(&self) -> CurrencyId {
        (*self).settlement_currency()
    }
}

fn linear_weight(
    instrument_id: InstrumentId,
    qty: Qty,
    multiplier: i64,
    market: &MarketSnapshot,
    now: Timestamp,
) -> RiskWeight {
    let price = match market.trusted_price(instrument_id, now) {
        Ok(price) => price,
        Err(reason) => return RiskWeight::Indeterminate(reason),
    };

    Notional::checked_linear(price, qty, multiplier).map_or(
        RiskWeight::Indeterminate(IndeterminateReason::ArithmeticOverflow),
        RiskWeight::Linear,
    )
}

/// Fixed-capacity currency set for v1 instruments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencySet {
    len: u8,
    items: [CurrencyId; 2],
}

impl CurrencySet {
    /// Creates a one-currency set.
    #[must_use]
    pub const fn one(currency: CurrencyId) -> Self {
        Self {
            len: 1,
            items: [currency, currency],
        }
    }

    /// Creates a two-currency set, collapsing duplicates.
    #[must_use]
    pub const fn two(first: CurrencyId, second: CurrencyId) -> Self {
        if first.0 == second.0 {
            Self::one(first)
        } else {
            Self {
                len: 2,
                items: [first, second],
            }
        }
    }

    /// Returns the number of distinct currencies.
    #[must_use]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    /// Returns whether the set is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Returns the currencies as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[CurrencyId] {
        &self.items[..self.len()]
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CurrencyId,
        instrument::{
            EquitySpec, Instrument, InstrumentCatalog, InstrumentCatalogError, OptionSpec,
            RiskExposure,
        },
        market::{MarketPrice, MarketSnapshot},
        types::{InstrumentId, Price, Qty, Timestamp},
        verdict::{IndeterminateReason, RiskWeight},
    };

    #[test]
    fn option_weight_is_indeterminate_in_v1() {
        let option = Instrument::Option(OptionSpec {
            instrument_id: InstrumentId(1),
            underlying_id: InstrumentId(2),
            settlement_currency: CurrencyId(840),
            expiry: Timestamp(1_000),
        });
        let market = MarketSnapshot::new(10, 10, 10);

        assert_eq!(
            option.risk_weight(Qty::new(10), &market, Timestamp(1)),
            RiskWeight::Indeterminate(IndeterminateReason::UnsupportedOption)
        );
    }

    #[test]
    fn equity_weight_uses_trusted_price() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(50), Timestamp(5)),
        );

        assert_eq!(
            equity.risk_weight(Qty::new(2), &market, Timestamp(10)),
            RiskWeight::Linear(crate::Notional::new(100))
        );
    }

    #[test]
    fn risk_exposure_trait_delegates_to_instrument() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(50), Timestamp(5)),
        );

        assert_eq!(
            RiskExposure::risk_weight(&equity, Qty::new(2), &market, Timestamp(10)),
            RiskWeight::Linear(crate::Notional::new(100))
        );
        assert_eq!(RiskExposure::settlement_currency(&equity), CurrencyId(840));
    }

    #[test]
    fn catalog_rejects_duplicate_instrument_ids() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let mut catalog = InstrumentCatalog::new();

        catalog.insert(equity).unwrap();

        assert_eq!(
            catalog.insert(equity),
            Err(InstrumentCatalogError::DuplicateInstrumentId(InstrumentId(
                1
            )))
        );
        assert_eq!(catalog.get(InstrumentId(1)), Some(equity));
        assert_eq!(catalog.len(), 1);
    }
}
