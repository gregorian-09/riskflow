//! Currency identifiers and pairs.

/// Copyable currency identifier used by risk calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrencyId(pub u16);

impl CurrencyId {
    /// Returns the raw numeric identifier.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Ordered base/quote currency pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurrencyPair {
    /// Base currency.
    pub base: CurrencyId,
    /// Quote currency.
    pub quote: CurrencyId,
}

impl CurrencyPair {
    /// Creates a currency pair.
    #[must_use]
    pub const fn new(base: CurrencyId, quote: CurrencyId) -> Self {
        Self { base, quote }
    }
}
