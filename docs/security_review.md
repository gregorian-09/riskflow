# External Review and Security Audit Packet

This packet defines what an external reviewer should inspect. It does not claim
that an independent audit has been completed; that requires a named reviewer
outside the implementation author.

## Scope

Review targets:

- fail-closed logic,
- fixed-point arithmetic and overflow handling,
- dependency boundary rules,
- file-backed limit parsing,
- audit records and operational state changes,
- optional Python binding surface,
- CI and dependency hardening.

Out of scope:

- options pricing,
- regulatory capital,
- live exchange integrations,
- deployment-specific secrets management.

## Required Reviewer Actions

1. Run the full quality gate.
2. Run `cargo audit --db target/advisory-db`.
3. Run `cargo deny check`.
4. Grep for unsafe code and confirm the workspace forbids it.
5. Review all parsers for malformed input handling.
6. Review all fixed-point arithmetic paths for checked operations.
7. Review dependency tree and advisory exceptions.
8. Review CI boundary checks for optional options-pricing implementation crates.

## Sign-Off Template

Use this table in an external review packet or release evidence bundle after a
named reviewer completes the required actions.

| Reviewer | Organization | Date | Scope | Decision | Notes |
|---|---|---|---|---|---|
|  |  |  |  |  |  |

## Evidence Sources

The repository contains:

- CI workflow for fmt, Clippy, tests, docs, audit, deny, and dependency
  boundary checks,
- `deny.toml` advisory and license policy,
- `docs/hardening.md` dependency timing, coverage, semver, and packaging notes,
- property tests for numeric primitives,
- adversarial tests for fail-closed pretrade behavior.
