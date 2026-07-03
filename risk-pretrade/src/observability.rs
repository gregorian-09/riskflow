//! Structured observability primitives for pretrade adapters.

use std::sync::atomic::{AtomicU64, Ordering};

use risk_core::{IndeterminateReason, RejectReason, RiskVerdict, Timestamp};

use crate::audit::{LimitChangeAuditRecord, OrderAuditRecord, TradingStateAuditRecord};

/// Atomic pretrade gate counters.
#[derive(Debug, Default)]
pub struct GateMetrics {
    evaluations: AtomicU64,
    passes: AtomicU64,
    rejects: AtomicU64,
    indeterminates: AtomicU64,
    trading_disabled_rejections: AtomicU64,
    limit_updates: AtomicU64,
    trading_state_changes: AtomicU64,
}

impl GateMetrics {
    /// Records one evaluated verdict.
    pub fn record_verdict(&self, verdict: RiskVerdict) {
        self.evaluations.fetch_add(1, Ordering::Relaxed);
        match verdict {
            RiskVerdict::Pass => {
                self.passes.fetch_add(1, Ordering::Relaxed);
            }
            RiskVerdict::Reject(RejectReason::TradingDisabled) => {
                self.rejects.fetch_add(1, Ordering::Relaxed);
                self.trading_disabled_rejections
                    .fetch_add(1, Ordering::Relaxed);
            }
            RiskVerdict::Reject(_) => {
                self.rejects.fetch_add(1, Ordering::Relaxed);
            }
            RiskVerdict::Indeterminate(_) => {
                self.indeterminates.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Records one limit table replacement.
    pub fn record_limit_update(&self) {
        self.limit_updates.fetch_add(1, Ordering::Relaxed);
    }

    /// Records one trading-enabled state change.
    pub fn record_trading_state_change(&self) {
        self.trading_state_changes.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns a point-in-time metrics snapshot.
    #[must_use]
    pub fn snapshot(&self) -> GateMetricsSnapshot {
        GateMetricsSnapshot {
            evaluations: self.evaluations.load(Ordering::Relaxed),
            passes: self.passes.load(Ordering::Relaxed),
            rejects: self.rejects.load(Ordering::Relaxed),
            indeterminates: self.indeterminates.load(Ordering::Relaxed),
            trading_disabled_rejections: self.trading_disabled_rejections.load(Ordering::Relaxed),
            limit_updates: self.limit_updates.load(Ordering::Relaxed),
            trading_state_changes: self.trading_state_changes.load(Ordering::Relaxed),
        }
    }
}

/// Exportable pretrade metrics snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GateMetricsSnapshot {
    /// Total evaluated orders.
    pub evaluations: u64,
    /// Orders accepted by all checks.
    pub passes: u64,
    /// Orders rejected by deterministic limits or operational controls.
    pub rejects: u64,
    /// Orders failed closed because the gate could not compute a trusted answer.
    pub indeterminates: u64,
    /// Rejections caused by disabled trading.
    pub trading_disabled_rejections: u64,
    /// Active limit table replacements.
    pub limit_updates: u64,
    /// Trading-enabled state transitions.
    pub trading_state_changes: u64,
}

/// Adapter-supplied trace context for correlating risk decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceContext {
    /// Adapter-local correlation identifier.
    pub correlation_id: u64,
    /// Monotonic adapter sequence number.
    pub sequence: u64,
    /// Timestamp at which the adapter created the event.
    pub observed_at: Timestamp,
}

/// Structured order-evaluation event suitable for logs or tracing exporters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedOrderEvent {
    /// Adapter trace context.
    pub trace: TraceContext,
    /// Auditable order decision payload.
    pub audit: OrderAuditRecord,
    /// Metrics snapshot captured after the decision.
    pub metrics: GateMetricsSnapshot,
}

impl ObservedOrderEvent {
    /// Builds a structured order-evaluation event.
    #[must_use]
    pub const fn new(
        trace: TraceContext,
        audit: OrderAuditRecord,
        metrics: GateMetricsSnapshot,
    ) -> Self {
        Self {
            trace,
            audit,
            metrics,
        }
    }
}

/// Structured limit-change event suitable for logs or tracing exporters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedLimitChangeEvent {
    /// Adapter trace context.
    pub trace: TraceContext,
    /// Auditable limit-change payload.
    pub audit: LimitChangeAuditRecord,
    /// Metrics snapshot captured after the change.
    pub metrics: GateMetricsSnapshot,
}

/// Structured trading-state event suitable for logs or tracing exporters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedTradingStateEvent {
    /// Adapter trace context.
    pub trace: TraceContext,
    /// Auditable trading-state payload.
    pub audit: TradingStateAuditRecord,
    /// Metrics snapshot captured after the change.
    pub metrics: GateMetricsSnapshot,
}

/// Operational alert severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    /// Informational event.
    Info,
    /// Operator-visible warning.
    Warning,
    /// Immediate action required.
    Critical,
}

/// Pretrade alert emitted from deterministic verdict categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PretradeAlert {
    /// Alert severity.
    pub severity: AlertSeverity,
    /// Rejection reason, if the alert came from a deterministic reject.
    pub reject_reason: Option<RejectReason>,
    /// Indeterminate reason, if the alert came from fail-closed uncertainty.
    pub indeterminate_reason: Option<IndeterminateReason>,
}

impl PretradeAlert {
    /// Maps a verdict to an alert, returning `None` for pass decisions.
    #[must_use]
    pub const fn from_verdict(verdict: RiskVerdict) -> Option<Self> {
        match verdict {
            RiskVerdict::Pass => None,
            RiskVerdict::Reject(reason) => Some(Self {
                severity: AlertSeverity::Warning,
                reject_reason: Some(reason),
                indeterminate_reason: None,
            }),
            RiskVerdict::Indeterminate(reason) => Some(Self {
                severity: AlertSeverity::Critical,
                reject_reason: None,
                indeterminate_reason: Some(reason),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use risk_core::{IndeterminateReason, RiskVerdict};

    use super::{AlertSeverity, GateMetrics, PretradeAlert};

    #[test]
    fn metrics_count_verdict_categories() {
        let metrics = GateMetrics::default();

        metrics.record_verdict(RiskVerdict::Pass);
        metrics.record_verdict(RiskVerdict::Indeterminate(
            IndeterminateReason::MissingPrice,
        ));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.evaluations, 2);
        assert_eq!(snapshot.passes, 1);
        assert_eq!(snapshot.indeterminates, 1);
    }

    #[test]
    fn indeterminate_verdict_maps_to_critical_alert() {
        let alert = PretradeAlert::from_verdict(RiskVerdict::Indeterminate(
            IndeterminateReason::StalePrice,
        ))
        .unwrap();

        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(
            alert.indeterminate_reason,
            Some(IndeterminateReason::StalePrice)
        );
    }
}
