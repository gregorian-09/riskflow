# Observability

The workspace keeps observability dependency-light in the core libraries. The
gate exposes structured records and metrics that adapters can export to JSON
logs, tracing spans, Prometheus, OpenTelemetry, or an internal telemetry bus.

## Metrics

`PretradeGate::metrics_snapshot()` returns:

- `evaluations`,
- `passes`,
- `rejects`,
- `indeterminates`,
- `trading_disabled_rejections`,
- `limit_updates`,
- `trading_state_changes`.

Adapters should publish these as monotonic counters. Alert thresholds are
deployment-specific, but indeterminate decisions should be treated as
operator-visible because they indicate fail-closed uncertainty.

## Structured Events

Use these structs for export:

- `OrderAuditRecord`,
- `LimitChangeAuditRecord`,
- `TradingStateAuditRecord`,
- `ObservedOrderEvent`,
- `ObservedLimitChangeEvent`,
- `ObservedTradingStateEvent`,
- `TraceContext`.

Every order-entry adapter should attach a correlation id and sequence number
through `TraceContext`, then export the resulting event after evaluation.

## Alerts

`PretradeAlert::from_verdict` maps:

- `Pass` to no alert,
- deterministic `Reject` to `Warning`,
- `Indeterminate` to `Critical`.

Production adapters may downgrade expected deterministic rejects at the edge,
but must not suppress indeterminate alerts without a documented desk policy.

## Dashboards

Minimum dashboard panels:

- pass/reject/indeterminate rate,
- indeterminate reason breakdown,
- trading-disabled state and disabled rejection count,
- limit update count and last update timestamp from audit records,
- market-data stale/bad-quality verdicts,
- p99.9 pretrade latency from the benchmark or live adapter measurement.
