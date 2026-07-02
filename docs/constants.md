# Constants and Fixture Values

The library avoids hidden policy values in production code. Values that are
part of financial convention, benchmark reporting, or deterministic simulation
are named constants in code.

## Production Constants

- `10_000` basis points per unit: used for source-agreement tolerance because
  risk price bands are configured in basis points.
- Standard normal z-scores for 90%, 95%, 97.5%, and 99%: used by parametric
  `VaR`; unsupported confidence levels return `None` instead of interpolation.
- Linear congruential generator multiplier and increment: used only to make
  Monte Carlo `VaR` deterministic for the same seed.
- 500 and 999 per-mille percentile markers: used by the benchmark report for
  median and p99.9 latency.

## Tests and Examples

Small literal ids, prices, quantities, and limits in tests are fixtures chosen
to make expected results easy to audit by hand. They are not default production
limits.
