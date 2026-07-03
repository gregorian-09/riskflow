# Operations Runbooks

These runbooks define the operational controls expected around `risk-pretrade`
before production use.

## Deployment

1. Build from a tagged commit with a clean tree.
2. Run the full quality gate from `README.md`.
3. Record crate versions, commit hash, Rust toolchain, and benchmark row.
4. Load reference data and build `SymbolRegistry` before order entry starts.
5. Load the active limit table from a versioned file.
6. Start the adapter with trading disabled.
7. Verify market snapshot freshness and aggregate notional availability.
8. Enable trading through `enable_trading_with_audit`.
9. Confirm metrics increment for a canary pass and a canary reject.

## Rollback

1. Disable trading through `disable_trading_with_audit`.
2. Preserve audit records and metrics snapshots.
3. Restore the previous binary, limit file, and reference-data snapshot.
4. Run the canary pass/reject checks against the restored version.
5. Re-enable trading only after the restored gate produces expected verdicts.
6. Attach the rollback audit records to the incident ticket.

## Limit-Change Approval

1. Limit changes require a named actor, ticket id, and effective timestamp.
2. The proposed file must include `schema_version,1,0,0`.
3. A second reviewer must compare old and new `LimitTableSummary` counts.
4. Apply the file through `update_limits_with_audit`.
5. Record the returned `LimitChangeAuditRecord`.
6. Monitor `limit_updates`, rejects, and indeterminates for the next interval.

## Incident Handling

Trigger an incident when any of the following occur:

- indeterminate verdicts exceed the desk threshold,
- bad data quality appears on active instruments,
- aggregate snapshot freshness is breached,
- trading is disabled unexpectedly,
- adapter symbol resolution fails for known production symbols,
- p99.9 latency exceeds the approved threshold.

Immediate actions:

1. Disable trading if fail-closed volume threatens order-entry availability.
2. Preserve structured order events and audit records.
3. Identify whether the source is market data, limits, reference data, or code.
4. Restore the last known good input snapshot if the binary is healthy.
5. Roll back the binary only after input-state causes are excluded.

## Release Evidence

Each production release should attach:

- commit hash and release tag,
- quality-gate output,
- benchmark matrix row,
- active schema versions,
- dependency audit output,
- model-validation sign-off status,
- security-review status.
