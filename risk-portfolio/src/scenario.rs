//! Deterministic scenario and stress testing helpers.

/// Return shock applied to a single asset in an ordered asset universe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScenarioShock {
    /// Zero-based asset index.
    pub asset_index: usize,
    /// Additive return shift, for example `-0.10` for a ten percent shock.
    pub return_shift: f64,
}

impl ScenarioShock {
    /// Creates a scenario shock.
    #[must_use]
    pub const fn new(asset_index: usize, return_shift: f64) -> Self {
        Self {
            asset_index,
            return_shift,
        }
    }
}

/// Result of applying a deterministic return scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioResult {
    /// Asset returns after shocks are applied.
    pub shocked_returns: Vec<f64>,
    /// Weighted portfolio return after shocks are applied.
    pub portfolio_return: f64,
    /// Positive loss amount, equal to `max(-portfolio_return, 0)`.
    pub portfolio_loss: f64,
}

/// Named scenario result useful for validation packs and reports.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedScenarioResult {
    /// Scenario name.
    pub name: String,
    /// Scenario result.
    pub result: ScenarioResult,
}

/// Deterministic named stress scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct StressScenario {
    /// Scenario name.
    pub name: String,
    /// Shocks applied by the scenario.
    pub shocks: Vec<ScenarioShock>,
}

impl StressScenario {
    /// Creates a stress scenario.
    #[must_use]
    pub fn new(name: impl Into<String>, shocks: Vec<ScenarioShock>) -> Self {
        Self {
            name: name.into(),
            shocks,
        }
    }
}

/// Applies additive return shocks and computes weighted portfolio loss.
#[must_use]
pub fn apply_return_shocks(
    base_returns: &[f64],
    weights: &[f64],
    shocks: &[ScenarioShock],
) -> Option<ScenarioResult> {
    if base_returns.is_empty()
        || base_returns.len() != weights.len()
        || base_returns.iter().any(|value| !value.is_finite())
        || weights.iter().any(|value| !value.is_finite())
        || shocks
            .iter()
            .any(|shock| shock.asset_index >= base_returns.len() || !shock.return_shift.is_finite())
    {
        return None;
    }

    let mut shocked_returns = base_returns.to_vec();
    for shock in shocks {
        shocked_returns[shock.asset_index] += shock.return_shift;
    }

    let portfolio_return = shocked_returns
        .iter()
        .zip(weights)
        .map(|(asset_return, weight)| asset_return * weight)
        .sum::<f64>();
    let portfolio_loss = (-portfolio_return).max(0.0);

    Some(ScenarioResult {
        shocked_returns,
        portfolio_return,
        portfolio_loss,
    })
}

/// Applies a list of named stress scenarios.
#[must_use]
pub fn run_stress_scenarios(
    base_returns: &[f64],
    weights: &[f64],
    scenarios: &[StressScenario],
) -> Option<Vec<NamedScenarioResult>> {
    scenarios
        .iter()
        .map(|scenario| {
            apply_return_shocks(base_returns, weights, &scenario.shocks).map(|result| {
                NamedScenarioResult {
                    name: scenario.name.clone(),
                    result,
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ScenarioShock, StressScenario, apply_return_shocks, run_stress_scenarios};

    #[test]
    fn applies_return_shocks_and_computes_loss() {
        let result =
            apply_return_shocks(&[0.01, 0.0], &[0.6, 0.4], &[ScenarioShock::new(0, -0.10)])
                .unwrap();

        assert!((result.shocked_returns[0] + 0.09).abs() < 1e-12);
        assert!(result.shocked_returns[1].abs() < 1e-12);
        assert!((result.portfolio_return + 0.054).abs() < 1e-12);
        assert!((result.portfolio_loss - 0.054).abs() < 1e-12);
    }

    #[test]
    fn rejects_invalid_shock_index() {
        assert_eq!(
            apply_return_shocks(&[0.01], &[1.0], &[ScenarioShock::new(1, -0.10)]),
            None
        );
    }

    #[test]
    fn runs_named_stress_scenarios() {
        let scenarios = [
            StressScenario::new("equity_down", vec![ScenarioShock::new(0, -0.10)]),
            StressScenario::new(
                "broad_riskoff",
                vec![ScenarioShock::new(0, -0.08), ScenarioShock::new(1, -0.04)],
            ),
        ];

        let results = run_stress_scenarios(&[0.01, 0.0], &[0.6, 0.4], &scenarios).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "equity_down");
        assert!((results[1].result.portfolio_loss - 0.058).abs() < 1e-12);
    }
}
