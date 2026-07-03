//! Compute a compact portfolio analytics report with `risk-portfolio`.

use nalgebra::dmatrix;
use risk_portfolio::{
    performance::summarize_returns,
    scenario::{ScenarioShock, StressScenario, try_run_stress_scenarios},
    var::{SimulationSeed, historical_var, monte_carlo_var, try_parametric_var_attribution},
};

fn main() {
    let returns = [0.03, -0.02, -0.10, 0.01, -0.05];
    let summary = summarize_returns(&returns, 0.0).expect("fixture returns are valid");
    let historical = historical_var(&returns, 0.80).expect("fixture VaR should compute");
    let monte_carlo =
        monte_carlo_var(0.0, 0.02, 0.95, 1_000, SimulationSeed(42)).expect("seeded VaR computes");

    let weights = [0.6, 0.4];
    let covariance = dmatrix![0.04, 0.01; 0.01, 0.09];
    let attribution = try_parametric_var_attribution(&weights, &covariance, 0.95)
        .expect("fixture attribution should compute");

    let scenarios = [StressScenario::new(
        "broad_riskoff",
        vec![ScenarioShock::new(0, -0.08), ScenarioShock::new(1, -0.04)],
    )];
    let stress = try_run_stress_scenarios(&[0.01, 0.0], &weights, &scenarios)
        .expect("fixture stress scenarios should compute");

    println!("mean_return={:.6}", summary.mean);
    println!("historical_var={historical:.6}");
    println!("monte_carlo_var={monte_carlo:.6}");
    println!("portfolio_var={:.6}", attribution.portfolio_var);
    println!("stress_loss={:.6}", stress[0].result.portfolio_loss);
}
