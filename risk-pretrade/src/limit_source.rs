//! Limit source boundary.

use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use risk_core::{InstrumentId, Notional, Qty};

use crate::gate::LimitTable;

/// Source of limit table updates.
pub trait LimitSource {
    /// Polls for a new limit table.
    fn poll_updates(&self) -> Option<LimitTable>;
}

/// Static v1 limit source.
#[derive(Debug, Clone)]
pub struct StaticLimitSource {
    limits: LimitTable,
}

impl StaticLimitSource {
    /// Creates a static limit source.
    #[must_use]
    pub const fn new(limits: LimitTable) -> Self {
        Self { limits }
    }
}

impl LimitSource for StaticLimitSource {
    fn poll_updates(&self) -> Option<LimitTable> {
        Some(self.limits.clone())
    }
}

/// File-backed v1 limit source.
#[derive(Debug, Clone)]
pub struct FileLimitSource {
    path: PathBuf,
}

impl FileLimitSource {
    /// Creates a file-backed limit source.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the configured file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl LimitSource for FileLimitSource {
    fn poll_updates(&self) -> Option<LimitTable> {
        let contents = fs::read_to_string(&self.path).ok()?;
        parse_limit_table(&contents).ok()
    }
}

/// Parses a v1 text limit table.
///
/// Supported records:
///
/// ```text
/// aggregate_notional,1000000
/// per_order_notional,1,10000
/// max_abs_position,1,500
/// fat_finger_band_bps,1,250
/// ```
///
/// Empty lines and `#` comments are ignored.
pub fn parse_limit_table(contents: &str) -> Result<LimitTable, ParseLimitTableError> {
    let mut limits = LimitTable::new();

    for (index, raw_line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
        match fields.as_slice() {
            ["aggregate_notional", raw_limit] => {
                limits.set_aggregate_notional(Notional::new(parse_i64(line_number, raw_limit)?));
            }
            ["per_order_notional", raw_instrument_id, raw_limit] => {
                limits.set_per_order_notional(
                    InstrumentId(parse_u32(line_number, raw_instrument_id)?),
                    Notional::new(parse_i64(line_number, raw_limit)?),
                );
            }
            ["max_abs_position", raw_instrument_id, raw_limit] => {
                limits.set_max_abs_position(
                    InstrumentId(parse_u32(line_number, raw_instrument_id)?),
                    Qty::new(parse_i64(line_number, raw_limit)?),
                );
            }
            ["fat_finger_band_bps", raw_instrument_id, raw_band_bps] => {
                limits.set_fat_finger_band_bps(
                    InstrumentId(parse_u32(line_number, raw_instrument_id)?),
                    parse_u32(line_number, raw_band_bps)?,
                );
            }
            [record, ..] => {
                return Err(ParseLimitTableError::new(
                    line_number,
                    ParseLimitTableErrorKind::InvalidRecord((*record).to_owned()),
                ));
            }
            [] => unreachable!("empty lines are skipped before splitting"),
        }
    }

    Ok(limits)
}

fn parse_i64(line: usize, raw: &str) -> Result<i64, ParseLimitTableError> {
    raw.parse::<i64>().map_err(|_| {
        ParseLimitTableError::new(
            line,
            ParseLimitTableErrorKind::InvalidInteger(raw.to_owned()),
        )
    })
}

fn parse_u32(line: usize, raw: &str) -> Result<u32, ParseLimitTableError> {
    raw.parse::<u32>().map_err(|_| {
        ParseLimitTableError::new(
            line,
            ParseLimitTableErrorKind::InvalidInteger(raw.to_owned()),
        )
    })
}

/// Error returned when parsing a text limit table fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLimitTableError {
    line: usize,
    kind: ParseLimitTableErrorKind,
}

impl ParseLimitTableError {
    /// Creates a parse error.
    #[must_use]
    pub const fn new(line: usize, kind: ParseLimitTableErrorKind) -> Self {
        Self { line, kind }
    }

    /// Returns the one-based line number.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the parse error kind.
    #[must_use]
    pub const fn kind(&self) -> &ParseLimitTableErrorKind {
        &self.kind
    }
}

impl fmt::Display for ParseLimitTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to parse limit table on line {}: {}",
            self.line, self.kind
        )
    }
}

impl Error for ParseLimitTableError {}

/// Detailed limit table parse error kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseLimitTableErrorKind {
    /// Unknown or malformed record.
    InvalidRecord(String),
    /// Invalid integer field.
    InvalidInteger(String),
}

impl fmt::Display for ParseLimitTableErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRecord(record) => write!(f, "invalid record `{record}`"),
            Self::InvalidInteger(value) => write!(f, "invalid integer `{value}`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use risk_core::{InstrumentId, Notional, Qty};

    use super::{FileLimitSource, LimitSource, ParseLimitTableErrorKind, parse_limit_table};

    #[test]
    fn parses_limit_table_text() {
        let limits = parse_limit_table(
            "
            # v1 static limits
            aggregate_notional,10000
            per_order_notional,1,1000
            max_abs_position,1,50
            fat_finger_band_bps,1,250
            ",
        )
        .unwrap();

        assert_eq!(
            limits.aggregate_notional_limit(),
            Some(Notional::new(10_000))
        );
        assert_eq!(
            limits.per_order_notional(InstrumentId(1)),
            Some(Notional::new(1_000))
        );
        assert_eq!(limits.max_abs_position(InstrumentId(1)), Some(Qty::new(50)));
        assert_eq!(limits.fat_finger_band_bps(InstrumentId(1)), Some(250));
    }

    #[test]
    fn rejects_invalid_record() {
        let error = parse_limit_table("unknown,1,2").unwrap_err();

        assert_eq!(error.line(), 1);
        assert_eq!(
            error.kind(),
            &ParseLimitTableErrorKind::InvalidRecord("unknown".to_owned())
        );
    }

    #[test]
    fn file_limit_source_loads_valid_file() {
        let path = temp_limit_path();
        fs::write(
            &path,
            "aggregate_notional,10000\nper_order_notional,1,1000\nmax_abs_position,1,50\nfat_finger_band_bps,1,250\n",
        )
        .unwrap();

        let source = FileLimitSource::new(path.clone());
        let limits = source.poll_updates().unwrap();

        assert_eq!(
            limits.per_order_notional(InstrumentId(1)),
            Some(Notional::new(1_000))
        );

        fs::remove_file(path).unwrap();
    }

    fn temp_limit_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("risk-pretrade-limits-{nanos}.txt"))
    }
}
