//! Limit source boundary.

use crate::gate::LimitTable;

/// Source of limit table updates.
pub trait LimitSource {
    /// Polls for a new limit table.
    fn poll_updates(&self) -> Option<LimitTable>;
}

/// Static v1 limit source.
#[derive(Debug, Clone)]
pub struct StaticLimitSource {
    limits: LimitTable,
}

impl StaticLimitSource {
    /// Creates a static limit source.
    #[must_use]
    pub const fn new(limits: LimitTable) -> Self {
        Self { limits }
    }
}

impl LimitSource for StaticLimitSource {
    fn poll_updates(&self) -> Option<LimitTable> {
        Some(self.limits.clone())
    }
}
