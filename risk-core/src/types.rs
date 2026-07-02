//! Fixed-point primitive types used by the risk workspace.

/// Stable, copyable instrument identity used inside risk hot paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstrumentId(pub u32);

impl InstrumentId {
    /// Returns the raw numeric identifier.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Fixed-point price value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(i64);

impl Price {
    /// Creates a price from its raw fixed-point representation.
    #[must_use]
    pub const fn new(raw: i64) -> Self {
        Self(raw)
    }

    /// Returns the raw fixed-point representation.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }
}

/// Fixed-point quantity value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Qty(i64);

impl Qty {
    /// Creates a quantity from its raw fixed-point representation.
    #[must_use]
    pub const fn new(raw: i64) -> Self {
        Self(raw)
    }

    /// Returns the raw fixed-point representation.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// Returns the absolute quantity, or `None` if `i64::MIN` would overflow.
    #[must_use]
    pub fn checked_abs(self) -> Option<Self> {
        self.0.checked_abs().map(Self)
    }

    /// Adds two quantities with overflow checking.
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }
}

/// Fixed-point notional value used for limit comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Notional(i64);

impl Notional {
    /// Creates a notional from its raw fixed-point representation.
    #[must_use]
    pub const fn new(raw: i64) -> Self {
        Self(raw)
    }

    /// Returns zero notional.
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Returns the raw fixed-point representation.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }

    /// Adds two notionals with overflow checking.
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    /// Computes `abs(price * qty * multiplier)` with overflow checking.
    #[must_use]
    pub fn checked_linear(price: Price, qty: Qty, multiplier: i64) -> Option<Self> {
        let raw = i128::from(price.raw())
            .checked_mul(i128::from(qty.raw()))?
            .checked_mul(i128::from(multiplier))?
            .checked_abs()?;

        i64::try_from(raw).ok().map(Self)
    }
}

/// Monotonic or wall-clock timestamp in nanoseconds, chosen by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Returns the raw timestamp value in nanoseconds.
    #[must_use]
    pub const fn nanos(self) -> u64 {
        self.0
    }

    /// Returns the saturating age between `self` and `now`.
    #[must_use]
    pub const fn age_at(self, now: Self) -> u64 {
        now.0.saturating_sub(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Notional, Price, Qty};

    #[test]
    fn checked_linear_rejects_overflow() {
        assert!(Notional::checked_linear(Price::new(i64::MAX), Qty::new(i64::MAX), 2).is_none());
    }

    #[test]
    fn checked_linear_returns_absolute_notional() {
        let notional = Notional::checked_linear(Price::new(10), Qty::new(-3), 2).unwrap();
        assert_eq!(notional.raw(), 60);
    }
}
