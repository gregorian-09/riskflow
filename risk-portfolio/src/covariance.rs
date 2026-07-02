//! Matrix-backed covariance analytics.

use nalgebra::DMatrix;

/// Computes a sample covariance matrix from aligned asset return series.
///
/// Each slice in `returns_by_asset` is one asset's periodic return series. All
/// assets must have the same number of observations, and at least two
/// observations are required for the sample denominator.
#[must_use]
pub fn sample_covariance_matrix(returns_by_asset: &[&[f64]]) -> Option<DMatrix<f64>> {
    let first = returns_by_asset.first()?;
    let periods = first.len();
    if periods < 2
        || returns_by_asset.iter().any(|returns| {
            returns.len() != periods || returns.iter().any(|value| !value.is_finite())
        })
    {
        return None;
    }

    let means = returns_by_asset
        .iter()
        .map(|returns| mean(returns))
        .collect::<Option<Vec<_>>>()?;
    let denominator = f64::from(u32::try_from(periods - 1).ok()?);
    let asset_count = returns_by_asset.len();
    let mut covariance = DMatrix::zeros(asset_count, asset_count);

    for row in 0..asset_count {
        for col in row..asset_count {
            let covariance_value = (0..periods)
                .map(|period| {
                    (returns_by_asset[row][period] - means[row])
                        * (returns_by_asset[col][period] - means[col])
                })
                .sum::<f64>()
                / denominator;

            covariance[(row, col)] = covariance_value;
            covariance[(col, row)] = covariance_value;
        }
    }

    Some(covariance)
}

fn mean(values: &[f64]) -> Option<f64> {
    let len = f64::from(u32::try_from(values.len()).ok()?);

    Some(values.iter().sum::<f64>() / len)
}

#[cfg(test)]
mod tests {
    use super::sample_covariance_matrix;

    #[test]
    fn covariance_matrix_is_symmetric() {
        let first = [0.01, 0.02, -0.01, 0.03];
        let second = [0.00, 0.01, -0.02, 0.04];
        let matrix = sample_covariance_matrix(&[&first, &second]).unwrap();

        assert_eq!(matrix.nrows(), 2);
        assert_eq!(matrix.ncols(), 2);
        assert!((matrix[(0, 1)] - matrix[(1, 0)]).abs() < 1e-12);
    }

    #[test]
    fn covariance_matrix_rejects_unaligned_series() {
        let first = [0.01, 0.02];
        let second = [0.00];

        assert_eq!(sample_covariance_matrix(&[&first, &second]), None);
    }
}
