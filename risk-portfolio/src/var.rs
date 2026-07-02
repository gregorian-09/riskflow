//! Value-at-risk analytics.

use crate::performance::{mean_return, sample_std_dev};

const CONFIDENCE_90: f64 = 0.90;
const CONFIDENCE_95: f64 = 0.95;
const CONFIDENCE_97_5: f64 = 0.975;
const CONFIDENCE_99: f64 = 0.99;
const Z_90: f64 = 1.281_551_565_544_600_4;
const Z_95: f64 = 1.644_853_626_951_472_2;
const Z_97_5: f64 = 1.959_963_984_540_054;
const Z_99: f64 = 2.326_347_874_040_840_8;
const CONFIDENCE_EPSILON: f64 = 1e-12;
const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const LCG_INCREMENT: u64 = 1;
const UPPER_32_SHIFT: u32 = 32;

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

/// Computes historical `VaR` as a positive loss amount from periodic returns.
#[must_use]
pub fn historical_var(returns: &[f64], confidence: f64) -> Option<f64> {
    if returns.is_empty() || !(0.0..1.0).contains(&confidence) {
        return None;
    }

    let mut sorted = returns.to_vec();
    sorted.sort_by(f64::total_cmp);

    let tail_probability = 1.0 - confidence;
    let len = u32::try_from(sorted.len()).ok()?;
    let raw_index = (tail_probability * f64::from(len)).floor();
    let mut index = 0_usize;
    while index + 1 < sorted.len() {
        let next = u32::try_from(index + 1).ok()?;
        if f64::from(next) > raw_index {
            break;
        }
        index += 1;
    }

    Some((-sorted[index]).max(0.0))
}

/// Computes parametric normal `VaR` using a fixed z-score approximation.
#[must_use]
pub fn parametric_var(returns: &[f64], confidence: f64) -> Option<f64> {
    let mean = mean_return(returns)?;
    let volatility = sample_std_dev(returns)?;
    let z = z_score(confidence)?;

    Some((z * volatility - mean).max(0.0))
}

/// Computes deterministic Monte Carlo `VaR` using a seeded pseudo-random stream.
#[must_use]
pub fn monte_carlo_var(
    mean: f64,
    volatility: f64,
    confidence: f64,
    samples: usize,
    seed: SimulationSeed,
) -> Option<f64> {
    if samples == 0 || volatility < 0.0 || !(0.0..1.0).contains(&confidence) {
        return None;
    }

    let mut rng = DeterministicRng::new(seed.raw());
    let mut simulated = Vec::with_capacity(samples);
    for _ in 0..samples {
        let normal = rng.standard_normal();
        simulated.push(mean + volatility * normal);
    }

    historical_var(&simulated, confidence)
}

fn z_score(confidence: f64) -> Option<f64> {
    let z = if (confidence - CONFIDENCE_90).abs() < CONFIDENCE_EPSILON {
        Z_90
    } else if (confidence - CONFIDENCE_95).abs() < CONFIDENCE_EPSILON {
        Z_95
    } else if (confidence - CONFIDENCE_97_5).abs() < CONFIDENCE_EPSILON {
        Z_97_5
    } else if (confidence - CONFIDENCE_99).abs() < CONFIDENCE_EPSILON {
        Z_99
    } else {
        return None;
    };

    Some(z)
}

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);
        self.state
    }

    fn next_unit(&mut self) -> f64 {
        let value = u32::try_from(self.next_u64() >> UPPER_32_SHIFT).unwrap_or(0);
        f64::from(value) * (1.0 / (f64::from(u32::MAX) + 1.0))
    }

    fn standard_normal(&mut self) -> f64 {
        let u1 = self.next_unit().max(f64::MIN_POSITIVE);
        let u2 = self.next_unit();
        (-2.0 * u1.ln()).sqrt() * (2.0 * core::f64::consts::PI * u2).cos()
    }
}

#[cfg(test)]
mod tests {
    use super::{SimulationSeed, historical_var, monte_carlo_var, parametric_var};

    #[test]
    fn historical_var_returns_tail_loss() {
        let returns = [0.03, -0.02, -0.10, 0.01, -0.05];

        assert_eq!(historical_var(&returns, 0.80), Some(0.10));
    }

    #[test]
    fn parametric_var_supports_standard_confidence_levels() {
        let returns = [-0.01, 0.0, 0.01];

        assert!(parametric_var(&returns, 0.95).is_some());
        assert_eq!(parametric_var(&returns, 0.93), None);
    }

    #[test]
    fn monte_carlo_var_is_seed_deterministic() {
        let first = monte_carlo_var(0.0, 0.02, 0.95, 1_000, SimulationSeed(42));
        let second = monte_carlo_var(0.0, 0.02, 0.95, 1_000, SimulationSeed(42));

        assert_eq!(first, second);
    }
}
