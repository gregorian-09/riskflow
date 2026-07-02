//! Startup-only symbol registry.

use std::{collections::HashMap, error::Error, fmt};

use crate::{types::InstrumentId, verdict::IndeterminateReason};

/// Allocating external symbol key used only outside the pretrade hot path.
pub use of_core::SymbolId as SymbolKey;

/// Symbol-to-instrument registry built before order evaluation starts.
#[derive(Debug, Clone, Default)]
pub struct SymbolRegistry {
    by_symbol: HashMap<SymbolKey, InstrumentId>,
    by_id: Vec<Option<SymbolKey>>,
}

impl SymbolRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a symbol with an explicit instrument id.
    pub fn register(
        &mut self,
        symbol: SymbolKey,
        instrument_id: InstrumentId,
    ) -> Result<(), RegisterSymbolError> {
        let id_index =
            usize::try_from(instrument_id.raw()).map_err(|_| RegisterSymbolError::IdTooLarge)?;

        if self.by_symbol.contains_key(&symbol) {
            return Err(RegisterSymbolError::DuplicateSymbol);
        }

        if self.by_id.len() <= id_index {
            self.by_id.resize_with(id_index + 1, || None);
        }

        if self.by_id[id_index].is_some() {
            return Err(RegisterSymbolError::DuplicateInstrumentId);
        }

        self.by_id[id_index] = Some(symbol.clone());
        self.by_symbol.insert(symbol, instrument_id);
        Ok(())
    }

    /// Resolves an external symbol to a hot-path-safe instrument id.
    pub fn resolve(&self, symbol: &SymbolKey) -> Result<InstrumentId, IndeterminateReason> {
        self.by_symbol
            .get(symbol)
            .copied()
            .ok_or(IndeterminateReason::UnknownSymbol)
    }

    /// Returns a symbol for audit/logging outside the hot path.
    #[must_use]
    pub fn symbol_for(&self, instrument_id: InstrumentId) -> Option<&SymbolKey> {
        let id_index = usize::try_from(instrument_id.raw()).ok()?;
        self.by_id.get(id_index).and_then(Option::as_ref)
    }
}

/// Symbol registration error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterSymbolError {
    /// Symbol was already registered.
    DuplicateSymbol,
    /// Instrument id was already registered.
    DuplicateInstrumentId,
    /// Instrument id cannot be represented as a local index.
    IdTooLarge,
}

impl fmt::Display for RegisterSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSymbol => f.write_str("symbol is already registered"),
            Self::DuplicateInstrumentId => f.write_str("instrument id is already registered"),
            Self::IdTooLarge => f.write_str("instrument id is too large"),
        }
    }
}

impl Error for RegisterSymbolError {}

#[cfg(test)]
mod tests {
    use super::{SymbolKey, SymbolRegistry};
    use crate::{types::InstrumentId, verdict::IndeterminateReason};

    #[test]
    fn registry_resolves_registered_symbol() {
        let mut registry = SymbolRegistry::new();
        let symbol = SymbolKey {
            venue: "XNYS".to_owned(),
            symbol: "IBM".to_owned(),
        };

        registry.register(symbol.clone(), InstrumentId(7)).unwrap();

        assert_eq!(registry.resolve(&symbol), Ok(InstrumentId(7)));
        assert_eq!(registry.symbol_for(InstrumentId(7)), Some(&symbol));
    }

    #[test]
    fn unknown_symbol_fails_closed() {
        let registry = SymbolRegistry::new();
        let symbol = SymbolKey {
            venue: "XNYS".to_owned(),
            symbol: "IBM".to_owned(),
        };

        assert_eq!(
            registry.resolve(&symbol),
            Err(IndeterminateReason::UnknownSymbol)
        );
    }
}
