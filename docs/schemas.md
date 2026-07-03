# Versioned Schemas and Migration Policy

External records must be versioned even when their current representation is a
simple text fixture. The Rust types in `risk-core::schema` define the current
record families and versions.

## Current Schemas

| Record Family | Constant | Version |
|---|---|---|
| Instrument reference | `INSTRUMENT_REFERENCE_SCHEMA` | `1.0.0` |
| Limit table | `LIMIT_TABLE_SCHEMA` | `1.0.0` |
| Audit record | `AUDIT_RECORD_SCHEMA` | `1.0.0` |
| Market snapshot | `MARKET_SNAPSHOT_SCHEMA` | `1.0.0` |
| Portfolio validation | `PORTFOLIO_VALIDATION_SCHEMA` | `1.0.0` |

The file-backed limit parser accepts:

```text
schema_version,1,0,0
```

Headerless v1 limit files remain accepted for compatibility with early
fixtures, but production files should include the schema header.

## Compatibility Rules

- Major version changes are breaking.
- Minor version changes are backward-compatible additions.
- Patch version changes are fixture, documentation, or editorial changes.
- A reader can consume records from the same major version when the writer's
  minor version is less than or equal to the reader's minor version.
- Unknown fields may only be introduced in a minor version if all production
  readers explicitly ignore them.

## Migration Procedure

1. Add the new schema version constant or bump the current constant.
2. Add parser support for the old and new schema where compatibility is
   promised.
3. Add fixtures for both versions.
4. Document the migration path in this file.
5. Run the full quality gate and preserve output with the release evidence.
6. Remove old-version support only in the next major schema version.
