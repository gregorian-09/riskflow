//! Python bindings for portfolio analytics.

#![allow(
    clippy::needless_pass_by_value,
    reason = "PyO3 extracts Python sequences into owned Vec values at the binding boundary"
)]

use pyo3::{exceptions::PyValueError, prelude::*};

use crate::{
    covariance::sample_covariance_matrix,
    performance::{
        calmar_ratio, cumulative_return, downside_deviation, max_drawdown, mean_return,
        sample_std_dev, sharpe_ratio, sortino_ratio,
    },
    var::{SimulationSeed, historical_var, monte_carlo_var, parametric_var},
};

/// Python module exposing portfolio analytics.
#[pymodule]
pub fn risk_portfolio(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(py_mean_return, module)?)?;
    module.add_function(wrap_pyfunction!(py_sample_std_dev, module)?)?;
    module.add_function(wrap_pyfunction!(py_downside_deviation, module)?)?;
    module.add_function(wrap_pyfunction!(py_sharpe_ratio, module)?)?;
    module.add_function(wrap_pyfunction!(py_sortino_ratio, module)?)?;
    module.add_function(wrap_pyfunction!(py_max_drawdown, module)?)?;
    module.add_function(wrap_pyfunction!(py_calmar_ratio, module)?)?;
    module.add_function(wrap_pyfunction!(py_cumulative_return, module)?)?;
    module.add_function(wrap_pyfunction!(py_historical_var, module)?)?;
    module.add_function(wrap_pyfunction!(py_parametric_var, module)?)?;
    module.add_function(wrap_pyfunction!(py_monte_carlo_var, module)?)?;
    module.add_function(wrap_pyfunction!(py_sample_covariance_matrix, module)?)?;
    Ok(())
}

#[pyfunction(name = "mean_return")]
fn py_mean_return(returns: Vec<f64>) -> PyResult<f64> {
    required(mean_return(&returns), "returns must not be empty")
}

#[pyfunction(name = "sample_std_dev")]
fn py_sample_std_dev(returns: Vec<f64>) -> PyResult<f64> {
    required(
        sample_std_dev(&returns),
        "at least two returns are required for sample standard deviation",
    )
}

#[pyfunction(name = "downside_deviation")]
fn py_downside_deviation(returns: Vec<f64>, minimum_acceptable_return: f64) -> PyResult<f64> {
    required(
        downside_deviation(&returns, minimum_acceptable_return),
        "returns must not be empty",
    )
}

#[pyfunction(name = "sharpe_ratio")]
fn py_sharpe_ratio(returns: Vec<f64>, risk_free_return: f64) -> PyResult<f64> {
    required(
        sharpe_ratio(&returns, risk_free_return),
        "returns must have non-zero finite sample volatility",
    )
}

#[pyfunction(name = "sortino_ratio")]
fn py_sortino_ratio(returns: Vec<f64>, minimum_acceptable_return: f64) -> PyResult<f64> {
    required(
        sortino_ratio(&returns, minimum_acceptable_return),
        "returns must have non-zero finite downside deviation",
    )
}

#[pyfunction(name = "max_drawdown")]
fn py_max_drawdown(returns: Vec<f64>) -> PyResult<f64> {
    required(max_drawdown(&returns), "returns must not be empty")
}

#[pyfunction(name = "calmar_ratio")]
fn py_calmar_ratio(returns: Vec<f64>) -> PyResult<f64> {
    required(
        calmar_ratio(&returns),
        "returns must have non-zero finite maximum drawdown",
    )
}

#[pyfunction(name = "cumulative_return")]
fn py_cumulative_return(returns: Vec<f64>) -> PyResult<f64> {
    required(cumulative_return(&returns), "returns must not be empty")
}

#[pyfunction(name = "historical_var")]
fn py_historical_var(returns: Vec<f64>, confidence: f64) -> PyResult<f64> {
    required(
        historical_var(&returns, confidence),
        "returns must not be empty and confidence must be in [0, 1)",
    )
}

#[pyfunction(name = "parametric_var")]
fn py_parametric_var(returns: Vec<f64>, confidence: f64) -> PyResult<f64> {
    required(
        parametric_var(&returns, confidence),
        "returns require sample volatility and confidence must be supported",
    )
}

#[pyfunction(name = "monte_carlo_var")]
fn py_monte_carlo_var(
    mean: f64,
    volatility: f64,
    confidence: f64,
    samples: usize,
    seed: u64,
) -> PyResult<f64> {
    required(
        monte_carlo_var(mean, volatility, confidence, samples, SimulationSeed(seed)),
        "samples must be positive, volatility non-negative, and confidence in [0, 1)",
    )
}

#[pyfunction(name = "sample_covariance_matrix")]
fn py_sample_covariance_matrix(returns_by_asset: Vec<Vec<f64>>) -> PyResult<Vec<Vec<f64>>> {
    let borrowed = returns_by_asset
        .iter()
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    let matrix = required(
        sample_covariance_matrix(&borrowed),
        "series must be aligned, finite, and contain at least two observations",
    )?;

    Ok((0..matrix.nrows())
        .map(|row| {
            (0..matrix.ncols())
                .map(|col| matrix[(row, col)])
                .collect::<Vec<_>>()
        })
        .collect())
}

fn required<T>(value: Option<T>, message: &'static str) -> PyResult<T> {
    value.ok_or_else(|| PyValueError::new_err(message))
}

#[cfg(test)]
mod tests {
    use super::{py_mean_return, py_sample_covariance_matrix};

    #[test]
    fn python_wrapper_returns_value() {
        assert!((py_mean_return(vec![1.0, 2.0, 3.0]).unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn python_wrapper_rejects_invalid_covariance_input() {
        let result = py_sample_covariance_matrix(vec![vec![0.01], vec![0.02, 0.03]]);

        assert!(result.is_err());
    }
}
