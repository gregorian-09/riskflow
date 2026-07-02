//! Performance analytics.

/// Summary performance statistics for a return series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerformanceSummary {
    /// Arithmetic mean return.
    pub mean: f64,
    /// Sample standard deviation.
    pub volatility: f64,
    /// Sharpe ratio using the supplied risk-free return.
    pub sharpe: Option<f64>,
    /// Sortino ratio using downside deviation.
    pub sortino: Option<f64>,
    /// Maximum drawdown from cumulative equity curve.
    pub max_drawdown: f64,
    /// Calmar ratio.
    pub calmar: Option<f64>,
}

/// Computes a performance summary for periodic returns.
#[must_use]
pub fn summarize_returns(returns: &[f64], risk_free_return: f64) -> Option<PerformanceSummary> {
    let mean = mean_return(returns)?;
    let volatility = sample_std_dev(returns)?;
    let max_drawdown = max_drawdown(returns)?;

    Some(PerformanceSummary {
        mean,
        volatility,
        sharpe: sharpe_ratio(returns, risk_free_return),
        sortino: sortino_ratio(returns, risk_free_return),
        max_drawdown,
        calmar: calmar_ratio(returns),
    })
}

/// Computes arithmetic mean for a slice of returns.
#[must_use]
pub fn mean_return(returns: &[f64]) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    let len = u32::try_from(returns.len()).ok()?;
    Some(returns.iter().sum::<f64>() / f64::from(len))
}

/// Computes sample standard deviation.
#[must_use]
pub fn sample_std_dev(returns: &[f64]) -> Option<f64> {
    if returns.len() < 2 {
        return None;
    }

    let mean = mean_return(returns)?;
    let variance_sum = returns
        .iter()
        .map(|value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f64>();
    let denominator = u32::try_from(returns.len() - 1).ok()?;

    Some((variance_sum / f64::from(denominator)).sqrt())
}

/// Computes downside deviation below a minimum acceptable return.
#[must_use]
pub fn downside_deviation(returns: &[f64], minimum_acceptable_return: f64) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    let downside_sum = returns
        .iter()
        .map(|value| (value - minimum_acceptable_return).min(0.0))
        .map(|downside| downside * downside)
        .sum::<f64>();
    let len = u32::try_from(returns.len()).ok()?;

    Some((downside_sum / f64::from(len)).sqrt())
}

/// Computes the Sharpe ratio.
#[must_use]
pub fn sharpe_ratio(returns: &[f64], risk_free_return: f64) -> Option<f64> {
    let excess_mean = mean_return(returns)? - risk_free_return;
    let volatility = sample_std_dev(returns)?;
    nonzero_div(excess_mean, volatility)
}

/// Computes the Sortino ratio.
#[must_use]
pub fn sortino_ratio(returns: &[f64], minimum_acceptable_return: f64) -> Option<f64> {
    let excess_mean = mean_return(returns)? - minimum_acceptable_return;
    let downside = downside_deviation(returns, minimum_acceptable_return)?;
    nonzero_div(excess_mean, downside)
}

/// Computes maximum drawdown from periodic returns.
#[must_use]
pub fn max_drawdown(returns: &[f64]) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    let mut equity = 1.0_f64;
    let mut peak = 1.0_f64;
    let mut max_drawdown = 0.0_f64;

    for value in returns {
        equity *= 1.0 + value;
        peak = peak.max(equity);
        let drawdown = if peak == 0.0 {
            0.0
        } else {
            (peak - equity) / peak
        };
        max_drawdown = max_drawdown.max(drawdown);
    }

    Some(max_drawdown)
}

/// Computes the Calmar ratio as total return divided by maximum drawdown.
#[must_use]
pub fn calmar_ratio(returns: &[f64]) -> Option<f64> {
    let total_return = cumulative_return(returns)?;
    let drawdown = max_drawdown(returns)?;
    nonzero_div(total_return, drawdown)
}

/// Computes cumulative compounded return.
#[must_use]
pub fn cumulative_return(returns: &[f64]) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    Some(
        returns
            .iter()
            .fold(1.0, |equity, value| equity * (1.0 + value))
            - 1.0,
    )
}

fn nonzero_div(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator == 0.0 || !denominator.is_finite() {
        None
    } else {
        Some(numerator / denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        calmar_ratio, max_drawdown, mean_return, sample_std_dev, sharpe_ratio, sortino_ratio,
    };

    #[test]
    fn mean_return_rejects_empty_input() {
        assert_eq!(mean_return(&[]), None);
    }

    #[test]
    fn mean_return_averages_values() {
        assert_eq!(mean_return(&[1.0, 2.0, 3.0]), Some(2.0));
    }

    #[test]
    fn sample_std_dev_uses_sample_denominator() {
        let std_dev = sample_std_dev(&[1.0, 2.0, 3.0]).unwrap();

        assert!((std_dev - 1.0).abs() < 1e-12);
    }

    #[test]
    fn ratios_and_drawdown_are_computed() {
        let returns = [0.10, -0.05, 0.02];

        assert!(sharpe_ratio(&returns, 0.0).is_some());
        assert!(sortino_ratio(&returns, 0.0).is_some());
        let drawdown = max_drawdown(&returns).unwrap();
        assert!((drawdown - 0.05).abs() < 1e-12);
        assert!(calmar_ratio(&returns).is_some());
    }
}
