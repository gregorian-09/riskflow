//! Offline portfolio analytics.
//!
//! Portfolio analytics are batch-oriented: allocation and `f64` statistics are
//! acceptable here, while settlement and pretrade limit math remain fixed-point
//! in `risk-core` and `risk-pretrade`.
//!
//! # Scope
//!
//! v1 includes historical, parametric, seeded Monte Carlo, marginal, and
//! component `VaR`, performance ratios, drawdown metrics, deterministic stress
//! scenarios, and trusted cross-currency netting helpers. Options Greeks are
//! not part of the v1 portfolio analytics surface.
//!
//! # Features
//!
//! - `python`: enables optional `pyo3` bindings for notebook consumers.
//!
//! # Historical And Monte Carlo `VaR`
//!
//! ```
//! use risk_portfolio::var::{SimulationSeed, historical_var, monte_carlo_var};
//!
//! let returns = [0.03, -0.02, -0.10, 0.01, -0.05];
//! let historical = historical_var(&returns, 0.80).unwrap();
//! let simulated = monte_carlo_var(0.0, 0.02, 0.95, 1_000, SimulationSeed(42)).unwrap();
//!
//! assert_eq!(historical, 0.10);
//! assert!(simulated >= 0.0);
//! ```
//!
//! # Performance Summary
//!
//! ```
//! use risk_portfolio::performance::summarize_returns;
//!
//! let returns = [0.01, -0.02, 0.03, 0.01, -0.01];
//! let summary = summarize_returns(&returns, 0.0).unwrap();
//!
//! assert!(summary.volatility >= 0.0);
//! assert!(summary.max_drawdown >= 0.0);
//! ```
//!
//! # Parametric Attribution
//!
//! ```
//! use nalgebra::dmatrix;
//! use risk_portfolio::var::try_parametric_var_attribution;
//!
//! let weights = [0.6, 0.4];
//! let covariance = dmatrix![0.04, 0.01; 0.01, 0.09];
//!
//! let report = try_parametric_var_attribution(&weights, &covariance, 0.95).unwrap();
//! let component_sum = report.component_var.iter().sum::<f64>();
//!
//! assert!((report.portfolio_var - component_sum).abs() < 1e-12);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod covariance;
pub mod greeks;
pub mod netting;
pub mod performance;
#[cfg(feature = "python")]
pub mod python;
pub mod scenario;
pub mod var;
