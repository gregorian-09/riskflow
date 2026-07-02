//! Audit records for pretrade gate operations.

use risk_core::{AssetClass, InstrumentId, Notional, Price, Qty, RiskVerdict, Timestamp};

use crate::gate::{EvaluateRequest, LimitTableSummary};

/// Audit record emitted for an evaluated order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderAuditRecord {
    /// Instrument identity evaluated by the gate.
    pub instrument_id: InstrumentId,
    /// Asset class evaluated by the gate.
    pub asset_class: AssetClass,
    /// Submitted order quantity.
    pub qty: Qty,
    /// Position before the order.
    pub current_position: Qty,
    /// Available margin supplied for the account or book.
    pub available_margin: Notional,
    /// Submitted order price.
    pub order_price: Price,
    /// Evaluation timestamp.
    pub evaluated_at: Timestamp,
    /// Final gate verdict.
    pub verdict: RiskVerdict,
}

impl OrderAuditRecord {
    /// Builds an order audit record from the request and verdict.
    #[must_use]
    pub fn from_evaluation(request: EvaluateRequest<'_>, verdict: RiskVerdict) -> Self {
        Self {
            instrument_id: request.instrument.id(),
            asset_class: request.instrument.asset_class(),
            qty: request.qty,
            current_position: request.current_position,
            available_margin: request.available_margin,
            order_price: request.order_price,
            evaluated_at: request.now,
            verdict,
        }
    }
}

/// Audit record emitted when the limit snapshot is replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitChangeAuditRecord {
    /// Operator, adapter, or process responsible for the change.
    pub actor: String,
    /// Change timestamp.
    pub changed_at: Timestamp,
    /// Previous limit-table shape.
    pub previous: LimitTableSummary,
    /// New limit-table shape.
    pub current: LimitTableSummary,
}

/// Audit record emitted when trading state changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TradingStateAuditRecord {
    /// Operator, adapter, or process responsible for the change.
    pub actor: String,
    /// Change timestamp.
    pub changed_at: Timestamp,
    /// Previous trading-enabled state.
    pub previous_enabled: bool,
    /// New trading-enabled state.
    pub current_enabled: bool,
}

/// Unified pretrade audit event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateAuditRecord {
    /// An order was evaluated.
    OrderEvaluation(OrderAuditRecord),
    /// The active limit snapshot changed.
    LimitChange(LimitChangeAuditRecord),
    /// The trading-enabled state changed.
    TradingStateChange(TradingStateAuditRecord),
}

/// In-memory audit collector useful for tests and adapters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryAuditLog {
    records: Vec<GateAuditRecord>,
}

impl InMemoryAuditLog {
    /// Creates an empty audit log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an audit record.
    pub fn push(&mut self, record: GateAuditRecord) {
        self.records.push(record);
    }

    /// Returns all records in insertion order.
    #[must_use]
    pub fn records(&self) -> &[GateAuditRecord] {
        &self.records
    }

    /// Consumes the log and returns the owned records.
    #[must_use]
    pub fn into_records(self) -> Vec<GateAuditRecord> {
        self.records
    }
}
