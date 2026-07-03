//! Version identifiers for external risk records.

/// Semantic schema version for serialized or external records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    /// Breaking schema version.
    pub major: u16,
    /// Backward-compatible schema addition version.
    pub minor: u16,
    /// Editorial or fixture-only schema patch version.
    pub patch: u16,
}

impl SchemaVersion {
    /// Creates a schema version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns whether `self` can read a record written with `writer`.
    ///
    /// Major versions must match. Readers accept records from the same major
    /// version when the writer's minor version is less than or equal to the
    /// reader's minor version.
    #[must_use]
    pub const fn can_read(self, writer: Self) -> bool {
        self.major == writer.major && self.minor >= writer.minor
    }
}

/// Externally versioned record families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaRecordKind {
    /// Static instrument reference data.
    InstrumentReference,
    /// Static or file-backed pretrade limits.
    LimitTable,
    /// Order and operator audit records.
    AuditRecord,
    /// Market snapshot inputs and aggregate exposure snapshots.
    MarketSnapshot,
    /// Portfolio validation fixture records.
    PortfolioValidation,
}

/// Current v1 instrument reference schema.
pub const INSTRUMENT_REFERENCE_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0, 0);
/// Current v1 limit table schema.
pub const LIMIT_TABLE_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0, 0);
/// Current v1 audit record schema.
pub const AUDIT_RECORD_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0, 0);
/// Current v1 market snapshot schema.
pub const MARKET_SNAPSHOT_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0, 0);
/// Current v1 portfolio validation fixture schema.
pub const PORTFOLIO_VALIDATION_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0, 0);

/// Versioned schema descriptor for a record family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaDescriptor {
    /// Record family.
    pub kind: SchemaRecordKind,
    /// Current schema version.
    pub version: SchemaVersion,
}

impl SchemaDescriptor {
    /// Creates a schema descriptor.
    #[must_use]
    pub const fn new(kind: SchemaRecordKind, version: SchemaVersion) -> Self {
        Self { kind, version }
    }
}

/// Returns the current schema descriptor for a record family.
#[must_use]
pub const fn current_schema(kind: SchemaRecordKind) -> SchemaDescriptor {
    let version = match kind {
        SchemaRecordKind::InstrumentReference => INSTRUMENT_REFERENCE_SCHEMA,
        SchemaRecordKind::LimitTable => LIMIT_TABLE_SCHEMA,
        SchemaRecordKind::AuditRecord => AUDIT_RECORD_SCHEMA,
        SchemaRecordKind::MarketSnapshot => MARKET_SNAPSHOT_SCHEMA,
        SchemaRecordKind::PortfolioValidation => PORTFOLIO_VALIDATION_SCHEMA,
    };

    SchemaDescriptor::new(kind, version)
}

#[cfg(test)]
mod tests {
    use super::{SchemaRecordKind, SchemaVersion, current_schema};

    #[test]
    fn readers_accept_older_minor_versions() {
        let reader = SchemaVersion::new(1, 2, 0);
        let writer = SchemaVersion::new(1, 1, 0);

        assert!(reader.can_read(writer));
    }

    #[test]
    fn readers_reject_newer_major_versions() {
        let reader = SchemaVersion::new(1, 2, 0);
        let writer = SchemaVersion::new(2, 0, 0);

        assert!(!reader.can_read(writer));
    }

    #[test]
    fn current_limit_schema_is_v1() {
        let schema = current_schema(SchemaRecordKind::LimitTable);

        assert_eq!(schema.version, SchemaVersion::new(1, 0, 0));
    }
}
