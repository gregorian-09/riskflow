//! Golden stress-scenario validation fixtures.

use std::{collections::BTreeMap, error::Error, fmt};

use risk_portfolio::scenario::{ScenarioShock, StressScenario, run_stress_scenarios};

const FIXTURES: &str = include_str!("fixtures/stress_scenarios.csv");
const BASE_RETURNS: [f64; 2] = [0.01, 0.0];
const WEIGHTS: [f64; 2] = [0.6, 0.4];
const FLOAT_EPSILON: f64 = 1e-12;

#[test]
fn golden_stress_losses_match() -> Result<(), FixtureError> {
    let scenarios = parse_scenarios()?;
    let stress_scenarios = scenarios
        .iter()
        .map(|(name, scenario)| StressScenario::new(name.clone(), scenario.shocks.clone()))
        .collect::<Vec<_>>();
    let results = run_stress_scenarios(&BASE_RETURNS, &WEIGHTS, &stress_scenarios)
        .ok_or(FixtureError::ScenarioEvaluation)?;

    for result in results {
        let expected = scenarios
            .get(&result.name)
            .ok_or_else(|| FixtureError::MissingScenario(result.name.clone()))?
            .expected_loss;

        assert!(
            (result.result.portfolio_loss - expected).abs() < FLOAT_EPSILON,
            "{}",
            result.name
        );
    }

    Ok(())
}

fn parse_scenarios() -> Result<BTreeMap<String, FixtureScenario>, FixtureError> {
    let mut scenarios = BTreeMap::<String, FixtureScenario>::new();

    for (index, line) in FIXTURES.lines().enumerate().skip(1) {
        let fields = line.split(',').collect::<Vec<_>>();
        let [name, raw_asset_index, raw_return_shift, raw_expected_loss] = fields.as_slice() else {
            return Err(FixtureError::InvalidRow(index + 1));
        };
        let expected_loss = parse_f64(index + 1, raw_expected_loss)?;
        let entry = scenarios
            .entry((*name).to_owned())
            .or_insert_with(|| FixtureScenario {
                shocks: Vec::new(),
                expected_loss,
            });

        if (entry.expected_loss - expected_loss).abs() >= FLOAT_EPSILON {
            return Err(FixtureError::InconsistentExpectedLoss(index + 1));
        }

        entry.shocks.push(ScenarioShock::new(
            parse_usize(index + 1, raw_asset_index)?,
            parse_f64(index + 1, raw_return_shift)?,
        ));
    }

    Ok(scenarios)
}

fn parse_usize(line: usize, value: &str) -> Result<usize, FixtureError> {
    value
        .parse()
        .map_err(|_| FixtureError::InvalidInteger(line, value.to_owned()))
}

fn parse_f64(line: usize, value: &str) -> Result<f64, FixtureError> {
    value
        .parse()
        .map_err(|_| FixtureError::InvalidFloat(line, value.to_owned()))
}

#[derive(Debug)]
struct FixtureScenario {
    shocks: Vec<ScenarioShock>,
    expected_loss: f64,
}

#[derive(Debug)]
enum FixtureError {
    InvalidRow(usize),
    InvalidInteger(usize, String),
    InvalidFloat(usize, String),
    InconsistentExpectedLoss(usize),
    ScenarioEvaluation,
    MissingScenario(String),
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRow(line) => write!(f, "invalid fixture row on line {line}"),
            Self::InvalidInteger(line, value) => {
                write!(f, "invalid integer `{value}` on line {line}")
            }
            Self::InvalidFloat(line, value) => write!(f, "invalid float `{value}` on line {line}"),
            Self::InconsistentExpectedLoss(line) => {
                write!(f, "inconsistent expected loss on line {line}")
            }
            Self::ScenarioEvaluation => f.write_str("scenario evaluation failed"),
            Self::MissingScenario(name) => write!(f, "missing scenario `{name}`"),
        }
    }
}

impl Error for FixtureError {}
