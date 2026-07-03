//! Golden historical `VaR` validation fixtures.

use std::{collections::BTreeMap, error::Error, fmt};

use risk_portfolio::var::try_historical_var;

const FIXTURES: &str = include_str!("fixtures/historical_var.csv");
const FLOAT_EPSILON: f64 = 1e-12;

#[test]
fn golden_historical_var_matches_longer_fixtures() -> Result<(), FixtureError> {
    let scenarios = parse_scenarios()?;

    for (name, scenario) in scenarios {
        let actual = try_historical_var(&scenario.returns, 0.95)
            .map_err(|_| FixtureError::ScenarioEvaluation(name.clone()))?;

        assert!(
            (actual - scenario.expected_var_95).abs() < FLOAT_EPSILON,
            "{name}"
        );
    }

    Ok(())
}

fn parse_scenarios() -> Result<BTreeMap<String, FixtureScenario>, FixtureError> {
    let mut scenarios = BTreeMap::<String, FixtureScenario>::new();

    for (index, line) in FIXTURES.lines().enumerate().skip(1) {
        let fields = line.split(',').collect::<Vec<_>>();
        let [name, raw_return, raw_expected] = fields.as_slice() else {
            return Err(FixtureError::InvalidRow(index + 1));
        };
        let expected_var_95 = parse_f64(index + 1, raw_expected)?;
        let entry = scenarios
            .entry((*name).to_owned())
            .or_insert_with(|| FixtureScenario {
                returns: Vec::new(),
                expected_var_95,
            });

        if (entry.expected_var_95 - expected_var_95).abs() >= FLOAT_EPSILON {
            return Err(FixtureError::InconsistentExpectedVar(index + 1));
        }

        entry.returns.push(parse_f64(index + 1, raw_return)?);
    }

    Ok(scenarios)
}

fn parse_f64(line: usize, value: &str) -> Result<f64, FixtureError> {
    value
        .parse()
        .map_err(|_| FixtureError::InvalidFloat(line, value.to_owned()))
}

#[derive(Debug)]
struct FixtureScenario {
    returns: Vec<f64>,
    expected_var_95: f64,
}

#[derive(Debug)]
enum FixtureError {
    InvalidRow(usize),
    InvalidFloat(usize, String),
    InconsistentExpectedVar(usize),
    ScenarioEvaluation(String),
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRow(line) => write!(f, "invalid fixture row on line {line}"),
            Self::InvalidFloat(line, value) => write!(f, "invalid float `{value}` on line {line}"),
            Self::InconsistentExpectedVar(line) => {
                write!(f, "inconsistent expected `VaR` on line {line}")
            }
            Self::ScenarioEvaluation(name) => write!(f, "scenario `{name}` failed evaluation"),
        }
    }
}

impl Error for FixtureError {}
