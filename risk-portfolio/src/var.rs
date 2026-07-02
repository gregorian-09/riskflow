//! Value-at-risk placeholders.

/// Seed value used by deterministic simulation-based analytics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulationSeed(pub u64);

impl SimulationSeed {
    /// Returns the raw seed value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
