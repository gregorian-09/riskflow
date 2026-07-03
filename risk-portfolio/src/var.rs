//! Value-at-risk analytics.

use std::{error::Error, fmt};

use crate::performance::{mean_return, sample_std_dev};
use nalgebra::{DMatrix, DVector};

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

/// Parametric `VaR` attribution report for an ordered asset universe.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricVarAttribution {
    /// Total zero-mean portfolio `VaR`.
    pub portfolio_var: f64,
    /// Derivative of portfolio `VaR` with respect to each asset weight.
    pub marginal_var: Vec<f64>,
    /// Per-asset component contribution, equal to `weight_i * marginal_var_i`.
    pub component_var: Vec<f64>,
}

/// Error returned by typed `VaR` analytics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarError {
    /// Input data is empty.
    EmptyInput,
    /// Confidence level is outside the supported set or invalid.
    InvalidConfidence,
    /// Input contains a non-finite value.
    NonFiniteInput,
    /// Weights and covariance dimensions do not describe the same asset set.
    ShapeMismatch,
    /// Portfolio volatility is zero, so marginal attribution is undefined.
    ZeroVolatility,
    /// Covariance produced a negative or non-finite portfolio variance.
    InvalidVariance,
}

impl fmt::Display for VarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("input data is empty"),
            Self::InvalidConfidence => f.write_str("confidence level is invalid or unsupported"),
            Self::NonFiniteInput => f.write_str("input contains a non-finite value"),
            Self::ShapeMismatch => f.write_str("weights and covariance dimensions do not match"),
            Self::ZeroVolatility => f.write_str("portfolio volatility is zero"),
            Self::InvalidVariance => {
                f.write_str("covariance produced an invalid portfolio variance")
            }
        }
    }
}

impl Error for VarError {}

/// Computes historical `VaR` as a positive loss amount from periodic returns.
#[must_use]
pub fn historical_var(returns: &[f64], confidence: f64) -> Option<f64> {
    try_historical_var(returns, confidence).ok()
}

/// Computes historical `VaR` as a positive loss amount from periodic returns.
///
/// This typed variant reports why a value cannot be computed.
pub fn try_historical_var(returns: &[f64], confidence: f64) -> Result<f64, VarError> {
    validate_returns(returns)?;
    validate_confidence(confidence)?;

    let mut sorted = returns.to_vec();
    sorted.sort_by(f64::total_cmp);

    let tail_probability = 1.0 - confidence;
    let len = u32::try_from(sorted.len()).map_err(|_| VarError::InvalidVariance)?;
    let raw_index = (tail_probability * f64::from(len)).floor();
    let mut index = 0_usize;
    while index + 1 < sorted.len() {
        let next = u32::try_from(index + 1).map_err(|_| VarError::InvalidVariance)?;
        if f64::from(next) > raw_index {
            break;
        }
        index += 1;
    }

    Ok((-sorted[index]).max(0.0))
}

/// Computes parametric normal `VaR` using a fixed z-score approximation.
#[must_use]
pub fn parametric_var(returns: &[f64], confidence: f64) -> Option<f64> {
    try_parametric_var(returns, confidence).ok()
}

/// Computes parametric normal `VaR` using a fixed z-score approximation.
///
/// This typed variant reports why a value cannot be computed.
pub fn try_parametric_var(returns: &[f64], confidence: f64) -> Result<f64, VarError> {
    validate_returns(returns)?;
    let mean = mean_return(returns).ok_or(VarError::EmptyInput)?;
    let volatility = sample_std_dev(returns).ok_or(VarError::InvalidVariance)?;
    let z = z_score(confidence)?;

    Ok((z * volatility - mean).max(0.0))
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
    try_monte_carlo_var(mean, volatility, confidence, samples, seed).ok()
}

/// Computes deterministic Monte Carlo `VaR` using a seeded pseudo-random stream.
///
/// This typed variant reports why a value cannot be computed.
pub fn try_monte_carlo_var(
    mean: f64,
    volatility: f64,
    confidence: f64,
    samples: usize,
    seed: SimulationSeed,
) -> Result<f64, VarError> {
    if samples == 0 {
        return Err(VarError::EmptyInput);
    }
    if !mean.is_finite() || !volatility.is_finite() {
        return Err(VarError::NonFiniteInput);
    }
    if volatility < 0.0 {
        return Err(VarError::InvalidVariance);
    }
    validate_confidence(confidence)?;

    let mut rng = DeterministicRng::new(seed.raw());
    let mut simulated = Vec::with_capacity(samples);
    for _ in 0..samples {
        let normal = rng.standard_normal();
        simulated.push(mean + volatility * normal);
    }

    try_historical_var(&simulated, confidence)
}

/// Computes marginal parametric `VaR` contributions for weighted assets.
///
/// `weights` and `covariance` must describe the same ordered asset universe.
/// The returned values are the derivative of portfolio `VaR` with respect to
/// each asset weight under a zero-mean normal approximation.
#[must_use]
pub fn marginal_parametric_var(
    weights: &[f64],
    covariance: &DMatrix<f64>,
    confidence: f64,
) -> Option<Vec<f64>> {
    try_marginal_parametric_var(weights, covariance, confidence).ok()
}

/// Computes marginal parametric `VaR` contributions for weighted assets.
///
/// This typed variant reports why attribution cannot be computed.
pub fn try_marginal_parametric_var(
    weights: &[f64],
    covariance: &DMatrix<f64>,
    confidence: f64,
) -> Result<Vec<f64>, VarError> {
    let sigma = portfolio_volatility(weights, covariance)?;
    if sigma == 0.0 {
        return Err(VarError::ZeroVolatility);
    }

    let z = z_score(confidence)?;
    let weights = DVector::from_column_slice(weights);
    let covariance_weight = covariance * &weights;

    Ok(covariance_weight
        .iter()
        .map(|value| z * value / sigma)
        .collect())
}

/// Computes component parametric `VaR` contributions for weighted assets.
///
/// Component contributions are `weight_i * marginal_var_i`; their sum equals
/// total zero-mean parametric `VaR` within floating-point tolerance.
#[must_use]
pub fn component_parametric_var(
    weights: &[f64],
    covariance: &DMatrix<f64>,
    confidence: f64,
) -> Option<Vec<f64>> {
    try_component_parametric_var(weights, covariance, confidence).ok()
}

/// Computes component parametric `VaR` contributions for weighted assets.
///
/// This typed variant reports why attribution cannot be computed.
pub fn try_component_parametric_var(
    weights: &[f64],
    covariance: &DMatrix<f64>,
    confidence: f64,
) -> Result<Vec<f64>, VarError> {
    let marginal = try_marginal_parametric_var(weights, covariance, confidence)?;

    Ok(weights
        .iter()
        .zip(marginal)
        .map(|(weight, marginal)| weight * marginal)
        .collect())
}

/// Computes zero-mean parametric portfolio `VaR` from weights and covariance.
#[must_use]
pub fn portfolio_parametric_var(
    weights: &[f64],
    covariance: &DMatrix<f64>,
    confidence: f64,
) -> Option<f64> {
    try_portfolio_parametric_var(weights, covariance, confidence).ok()
}

/// Computes zero-mean parametric portfolio `VaR` from weights and covariance.
///
/// This typed variant reports why the value cannot be computed.
pub fn try_portfolio_parametric_var(
    weights: &[f64],
    covariance: &DMatrix<f64>,
    confidence: f64,
) -> Result<f64, VarError> {
    Ok(z_score(confidence)? * portfolio_volatility(weights, covariance)?)
}

/// Computes a complete zero-mean parametric `VaR` attribution report.
pub fn try_parametric_var_attribution(
    weights: &[f64],
    covariance: &DMatrix<f64>,
    confidence: f64,
) -> Result<ParametricVarAttribution, VarError> {
    let portfolio_var = try_portfolio_parametric_var(weights, covariance, confidence)?;
    let marginal_var = try_marginal_parametric_var(weights, covariance, confidence)?;
    let component_var = weights
        .iter()
        .zip(&marginal_var)
        .map(|(weight, marginal)| weight * marginal)
        .collect();

    Ok(ParametricVarAttribution {
        portfolio_var,
        marginal_var,
        component_var,
    })
}

fn validate_returns(returns: &[f64]) -> Result<(), VarError> {
    if returns.is_empty() {
        return Err(VarError::EmptyInput);
    }
    if returns.iter().any(|value| !value.is_finite()) {
        return Err(VarError::NonFiniteInput);
    }

    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), VarError> {
    if !(0.0..1.0).contains(&confidence) {
        return Err(VarError::InvalidConfidence);
    }

    Ok(())
}

fn portfolio_volatility(weights: &[f64], covariance: &DMatrix<f64>) -> Result<f64, VarError> {
    if weights.is_empty() {
        return Err(VarError::EmptyInput);
    }
    if covariance.nrows() != weights.len() || covariance.ncols() != weights.len() {
        return Err(VarError::ShapeMismatch);
    }
    if weights.iter().any(|value| !value.is_finite())
        || covariance.iter().any(|value| !value.is_finite())
    {
        return Err(VarError::NonFiniteInput);
    }

    let weights = DVector::from_column_slice(weights);
    let variance = weights.transpose() * covariance * weights;
    let variance = variance[(0, 0)];
    if variance < 0.0 || !variance.is_finite() {
        return Err(VarError::InvalidVariance);
    }

    Ok(variance.sqrt())
}

fn z_score(confidence: f64) -> Result<f64, VarError> {
    let z = if (confidence - CONFIDENCE_90).abs() < CONFIDENCE_EPSILON {
        Z_90
    } else if (confidence - CONFIDENCE_95).abs() < CONFIDENCE_EPSILON {
        Z_95
    } else if (confidence - CONFIDENCE_97_5).abs() < CONFIDENCE_EPSILON {
        Z_97_5
    } else if (confidence - CONFIDENCE_99).abs() < CONFIDENCE_EPSILON {
        Z_99
    } else {
        return Err(VarError::InvalidConfidence);
    };

    Ok(z)
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
    use nalgebra::dmatrix;

    use super::{
        SimulationSeed, VarError, component_parametric_var, historical_var,
        marginal_parametric_var, monte_carlo_var, parametric_var, portfolio_parametric_var,
        try_marginal_parametric_var, try_parametric_var_attribution,
    };

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

    #[test]
    fn component_var_sums_to_portfolio_var() {
        let covariance = dmatrix![0.04, 0.01; 0.01, 0.09];
        let weights = [0.6, 0.4];

        let portfolio = portfolio_parametric_var(&weights, &covariance, 0.95).unwrap();
        let components = component_parametric_var(&weights, &covariance, 0.95).unwrap();
        let component_sum = components.iter().sum::<f64>();

        assert!((portfolio - component_sum).abs() < 1e-12);
    }

    #[test]
    fn marginal_var_rejects_shape_mismatch() {
        let covariance = dmatrix![0.04, 0.01; 0.01, 0.09];

        assert_eq!(marginal_parametric_var(&[0.6], &covariance, 0.95), None);
        assert_eq!(
            try_marginal_parametric_var(&[0.6], &covariance, 0.95),
            Err(VarError::ShapeMismatch)
        );
    }

    #[test]
    fn attribution_report_contains_consistent_components() {
        let covariance = dmatrix![0.04, 0.01; 0.01, 0.09];
        let weights = [0.6, 0.4];

        let report = try_parametric_var_attribution(&weights, &covariance, 0.95).unwrap();
        let component_sum = report.component_var.iter().sum::<f64>();

        assert_eq!(report.marginal_var.len(), 2);
        assert_eq!(report.component_var.len(), 2);
        assert!((report.portfolio_var - component_sum).abs() < 1e-12);
    }
}
