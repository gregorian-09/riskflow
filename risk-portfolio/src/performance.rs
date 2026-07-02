//! Performance analytics placeholders.

/// Computes arithmetic mean for a slice of returns.
#[must_use]
pub fn mean_return(returns: &[f64]) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    let len = u32::try_from(returns.len()).ok()?;
    Some(returns.iter().sum::<f64>() / f64::from(len))
}

#[cfg(test)]
mod tests {
    use super::mean_return;

    #[test]
    fn mean_return_rejects_empty_input() {
        assert_eq!(mean_return(&[]), None);
    }

    #[test]
    fn mean_return_averages_values() {
        assert_eq!(mean_return(&[1.0, 2.0, 3.0]), Some(2.0));
    }
}
