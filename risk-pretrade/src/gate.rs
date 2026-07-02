//! Pretrade gate and limit table.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use arc_swap::ArcSwap;
use risk_core::{
    Instrument, InstrumentId, MarketSnapshot, Notional, Price, Qty, RejectReason, RiskVerdict,
    RiskWeight, Timestamp,
};

use crate::audit::{LimitChangeAuditRecord, OrderAuditRecord, TradingStateAuditRecord};
use crate::checks::{aggregate_notional, fat_finger, margin, notional, position_limit};

/// Immutable pretrade limit table.
#[derive(Debug, Clone, Default)]
pub struct LimitTable {
    per_order_notional: HashMap<InstrumentId, Notional>,
    aggregate_notional: Option<Notional>,
    max_abs_position: HashMap<InstrumentId, Qty>,
    fat_finger_band_bps: HashMap<InstrumentId, u32>,
    initial_margin_per_unit: HashMap<InstrumentId, Notional>,
}

impl LimitTable {
    /// Creates an empty limit table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a per-order notional limit.
    pub fn set_per_order_notional(&mut self, instrument_id: InstrumentId, limit: Notional) {
        self.per_order_notional.insert(instrument_id, limit);
    }

    /// Sets the aggregate base-currency notional limit.
    pub const fn set_aggregate_notional(&mut self, limit: Notional) {
        self.aggregate_notional = Some(limit);
    }

    /// Sets a maximum absolute post-order position.
    pub fn set_max_abs_position(&mut self, instrument_id: InstrumentId, limit: Qty) {
        self.max_abs_position.insert(instrument_id, limit);
    }

    /// Sets a fat-finger price band in basis points.
    pub fn set_fat_finger_band_bps(&mut self, instrument_id: InstrumentId, band_bps: u32) {
        self.fat_finger_band_bps.insert(instrument_id, band_bps);
    }

    /// Sets initial margin requirement per absolute quantity unit.
    pub fn set_initial_margin_per_unit(&mut self, instrument_id: InstrumentId, margin: Notional) {
        self.initial_margin_per_unit.insert(instrument_id, margin);
    }

    /// Returns the per-order notional limit for an instrument.
    #[must_use]
    pub fn per_order_notional(&self, instrument_id: InstrumentId) -> Option<Notional> {
        self.per_order_notional.get(&instrument_id).copied()
    }

    /// Returns the aggregate base-currency notional limit.
    #[must_use]
    pub const fn aggregate_notional_limit(&self) -> Option<Notional> {
        self.aggregate_notional
    }

    /// Returns the maximum absolute post-order position for an instrument.
    #[must_use]
    pub fn max_abs_position(&self, instrument_id: InstrumentId) -> Option<Qty> {
        self.max_abs_position.get(&instrument_id).copied()
    }

    /// Returns the fat-finger band in basis points for an instrument.
    #[must_use]
    pub fn fat_finger_band_bps(&self, instrument_id: InstrumentId) -> Option<u32> {
        self.fat_finger_band_bps.get(&instrument_id).copied()
    }

    /// Returns initial margin requirement per absolute quantity unit.
    #[must_use]
    pub fn initial_margin_per_unit(&self, instrument_id: InstrumentId) -> Option<Notional> {
        self.initial_margin_per_unit.get(&instrument_id).copied()
    }

    /// Returns a shape summary suitable for operational audit logs.
    #[must_use]
    pub fn summary(&self) -> LimitTableSummary {
        LimitTableSummary {
            per_order_notional_count: self.per_order_notional.len(),
            aggregate_notional_configured: self.aggregate_notional.is_some(),
            max_abs_position_count: self.max_abs_position.len(),
            fat_finger_band_count: self.fat_finger_band_bps.len(),
            initial_margin_count: self.initial_margin_per_unit.len(),
        }
    }
}

/// Audit-safe shape summary of a limit table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitTableSummary {
    /// Number of per-order notional limits.
    pub per_order_notional_count: usize,
    /// Whether an aggregate notional limit is configured.
    pub aggregate_notional_configured: bool,
    /// Number of maximum-position limits.
    pub max_abs_position_count: usize,
    /// Number of fat-finger bands.
    pub fat_finger_band_count: usize,
    /// Number of initial-margin entries.
    pub initial_margin_count: usize,
}

/// Request evaluated by the pretrade gate.
#[derive(Debug, Clone, Copy)]
pub struct EvaluateRequest<'a> {
    /// Static instrument reference data.
    pub instrument: Instrument,
    /// Order quantity.
    pub qty: Qty,
    /// Current signed position before the order.
    pub current_position: Qty,
    /// Available margin for this account/book.
    pub available_margin: Notional,
    /// Submitted order price.
    pub order_price: Price,
    /// Market snapshot.
    pub market: &'a MarketSnapshot,
    /// Current timestamp.
    pub now: Timestamp,
}

/// Synchronous pretrade risk gate.
#[derive(Debug)]
pub struct PretradeGate {
    limits: ArcSwap<LimitTable>,
    trading_enabled: AtomicBool,
}

impl PretradeGate {
    /// Creates a gate from an immutable limit table snapshot.
    #[must_use]
    pub fn new(limits: LimitTable) -> Self {
        Self {
            limits: ArcSwap::from_pointee(limits),
            trading_enabled: AtomicBool::new(true),
        }
    }

    /// Replaces the active limit table snapshot.
    pub fn update_limits(&self, limits: LimitTable) {
        self.limits.store(Arc::new(limits));
    }

    /// Replaces the active limit table snapshot and returns an audit record.
    pub fn update_limits_with_audit(
        &self,
        limits: LimitTable,
        actor: impl Into<String>,
        changed_at: Timestamp,
    ) -> LimitChangeAuditRecord {
        let previous = self.limits.load().summary();
        self.update_limits(limits);
        let current = self.limits.load().summary();

        LimitChangeAuditRecord {
            actor: actor.into(),
            changed_at,
            previous,
            current,
        }
    }

    /// Returns whether order evaluation is currently enabled.
    #[must_use]
    pub fn trading_enabled(&self) -> bool {
        self.trading_enabled.load(Ordering::Acquire)
    }

    /// Sets the trading-enabled state and returns an audit record.
    pub fn set_trading_enabled_with_audit(
        &self,
        enabled: bool,
        actor: impl Into<String>,
        changed_at: Timestamp,
    ) -> TradingStateAuditRecord {
        let previous_enabled = self.trading_enabled.swap(enabled, Ordering::AcqRel);

        TradingStateAuditRecord {
            actor: actor.into(),
            changed_at,
            previous_enabled,
            current_enabled: enabled,
        }
    }

    /// Disables order evaluation and returns an audit record.
    pub fn disable_trading_with_audit(
        &self,
        actor: impl Into<String>,
        changed_at: Timestamp,
    ) -> TradingStateAuditRecord {
        self.set_trading_enabled_with_audit(false, actor, changed_at)
    }

    /// Enables order evaluation and returns an audit record.
    pub fn enable_trading_with_audit(
        &self,
        actor: impl Into<String>,
        changed_at: Timestamp,
    ) -> TradingStateAuditRecord {
        self.set_trading_enabled_with_audit(true, actor, changed_at)
    }

    /// Evaluates an order request and returns an audit record with the verdict.
    #[must_use]
    pub fn evaluate_with_audit(
        &self,
        request: EvaluateRequest<'_>,
    ) -> (RiskVerdict, OrderAuditRecord) {
        let verdict = self.evaluate(request);
        let audit = OrderAuditRecord::from_evaluation(request, verdict);

        (verdict, audit)
    }

    /// Evaluates an order request.
    #[must_use]
    pub fn evaluate(&self, request: EvaluateRequest<'_>) -> RiskVerdict {
        if !self.trading_enabled() {
            return RiskVerdict::Reject(RejectReason::TradingDisabled);
        }

        let limits = self.limits.load();
        let limits = &**limits;

        match request
            .instrument
            .risk_weight(request.qty, request.market, request.now)
        {
            RiskWeight::Linear(order_notional) => {
                let instrument_id = request.instrument.id();

                let verdict = notional::check_per_order(limits, instrument_id, order_notional);
                if !verdict.is_pass() {
                    return verdict;
                }

                let verdict =
                    aggregate_notional::check(limits, request.market, request.now, order_notional);
                if !verdict.is_pass() {
                    return verdict;
                }

                let verdict = position_limit::check(
                    limits,
                    instrument_id,
                    request.current_position,
                    request.qty,
                );
                if !verdict.is_pass() {
                    return verdict;
                }

                let verdict = margin::check(
                    limits,
                    request.instrument,
                    request.current_position,
                    request.qty,
                    request.available_margin,
                );
                if !verdict.is_pass() {
                    return verdict;
                }

                fat_finger::check(
                    limits,
                    instrument_id,
                    request.order_price,
                    request.market,
                    request.now,
                )
            }
            RiskWeight::Indeterminate(reason) => RiskVerdict::Indeterminate(reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use risk_core::{
        CurrencyId, DataQuality, EquitySpec, IndeterminateReason, Instrument, InstrumentId,
        MarketPrice, MarketSnapshot, Notional, OptionSpec, Price, Qty, RiskVerdict, Timestamp,
    };

    use super::{EvaluateRequest, LimitTable, PretradeGate};
    use crate::audit::OrderAuditRecord;

    fn limits(per_order: i64) -> LimitTable {
        let mut limits = LimitTable::new();
        limits.set_per_order_notional(InstrumentId(1), Notional::new(per_order));
        limits.set_aggregate_notional(Notional::new(10_000));
        limits.set_max_abs_position(InstrumentId(1), Qty::new(100));
        limits.set_fat_finger_band_bps(InstrumentId(1), 500);
        limits.set_initial_margin_per_unit(InstrumentId(1), Notional::new(10));
        limits
    }

    fn market(
        reference_price: i64,
        aggregate_notional: i64,
        observed_at: Timestamp,
    ) -> MarketSnapshot {
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(reference_price), observed_at),
        );
        market.set_aggregate_notional(
            Notional::new(aggregate_notional),
            observed_at,
            DataQuality::clean(),
        );
        market
    }

    #[test]
    fn option_order_is_indeterminate_without_risk_options_dependency() {
        let option = Instrument::Option(OptionSpec {
            instrument_id: InstrumentId(1),
            underlying_id: InstrumentId(2),
            settlement_currency: CurrencyId(840),
            expiry: Timestamp(1_000),
        });
        let gate = PretradeGate::new(limits(1_000));
        let market = market(100, 0, Timestamp(1));

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: option,
            qty: Qty::new(1),
            current_position: Qty::new(0),
            available_margin: Notional::new(1_000),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(1),
        });

        assert!(matches!(verdict, RiskVerdict::Indeterminate(_)));
    }

    #[test]
    fn linear_order_passes_within_limit() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let gate = PretradeGate::new(limits(1_000));
        let market = market(100, 0, Timestamp(5));

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            current_position: Qty::new(0),
            available_margin: Notional::new(1_000),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(10),
        });

        assert_eq!(verdict, RiskVerdict::Pass);
    }

    #[test]
    fn disabled_gate_rejects_with_audit_record() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let gate = PretradeGate::new(limits(1_000));
        let state_record = gate.disable_trading_with_audit("risk-manager", Timestamp(9));
        let market = market(100, 0, Timestamp(5));

        let (verdict, audit) = gate.evaluate_with_audit(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            current_position: Qty::new(0),
            available_margin: Notional::new(1_000),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(10),
        });

        assert!(state_record.previous_enabled);
        assert!(!state_record.current_enabled);
        assert_eq!(
            verdict,
            RiskVerdict::Reject(risk_core::RejectReason::TradingDisabled)
        );
        assert_eq!(
            audit,
            OrderAuditRecord {
                instrument_id: InstrumentId(1),
                asset_class: risk_core::AssetClass::Equity,
                qty: Qty::new(5),
                current_position: Qty::new(0),
                available_margin: Notional::new(1_000),
                order_price: Price::new(100),
                evaluated_at: Timestamp(10),
                verdict,
            }
        );
    }

    #[test]
    fn limit_update_returns_audit_summary() {
        let gate = PretradeGate::new(LimitTable::new());
        let record = gate.update_limits_with_audit(limits(1_000), "file-source", Timestamp(12));

        assert_eq!(record.actor, "file-source");
        assert_eq!(record.changed_at, Timestamp(12));
        assert_eq!(record.previous.per_order_notional_count, 0);
        assert_eq!(record.current.per_order_notional_count, 1);
        assert!(record.current.aggregate_notional_configured);
    }

    #[test]
    fn linear_order_rejects_above_limit() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let gate = PretradeGate::new(limits(100));
        let market = market(100, 0, Timestamp(5));

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            current_position: Qty::new(0),
            available_margin: Notional::new(1_000),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(10),
        });

        assert!(matches!(verdict, RiskVerdict::Reject(_)));
    }

    #[test]
    fn linear_order_rejects_above_position_limit() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let mut limits = limits(1_000);
        limits.set_max_abs_position(InstrumentId(1), Qty::new(10));
        limits.set_fat_finger_band_bps(InstrumentId(1), 500);
        let gate = PretradeGate::new(limits);
        let market = market(100, 0, Timestamp(5));

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            current_position: Qty::new(7),
            available_margin: Notional::new(1_000),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(10),
        });

        assert!(matches!(verdict, RiskVerdict::Reject(_)));
    }

    #[test]
    fn linear_order_rejects_outside_fat_finger_band() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let gate = PretradeGate::new(limits(1_000));
        let market = market(100, 0, Timestamp(5));

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            current_position: Qty::new(0),
            available_margin: Notional::new(1_000),
            order_price: Price::new(120),
            market: &market,
            now: Timestamp(10),
        });

        assert!(matches!(verdict, RiskVerdict::Reject(_)));
    }

    #[test]
    fn linear_order_rejects_above_aggregate_notional_limit() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let mut limits = limits(1_000);
        limits.set_aggregate_notional(Notional::new(700));
        let gate = PretradeGate::new(limits);
        let market = market(100, 300, Timestamp(5));

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            current_position: Qty::new(0),
            available_margin: Notional::new(1_000),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(10),
        });

        assert!(matches!(verdict, RiskVerdict::Reject(_)));
    }

    #[test]
    fn stale_aggregate_snapshot_fails_closed() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let gate = PretradeGate::new(limits(1_000));
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(100), Timestamp(15)),
        );
        market.set_aggregate_notional(Notional::new(0), Timestamp(5), DataQuality::clean());

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            current_position: Qty::new(0),
            available_margin: Notional::new(1_000),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(20),
        });

        assert_eq!(
            verdict,
            RiskVerdict::Indeterminate(IndeterminateReason::StaleAggregateSnapshot)
        );
    }

    #[test]
    fn future_order_rejects_when_initial_margin_exceeds_available_margin() {
        let future = Instrument::Future(risk_core::FutureSpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
            multiplier: 1,
        });
        let gate = PretradeGate::new(limits(1_000));
        let market = market(100, 0, Timestamp(5));

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: future,
            qty: Qty::new(5),
            current_position: Qty::new(0),
            available_margin: Notional::new(40),
            order_price: Price::new(100),
            market: &market,
            now: Timestamp(10),
        });

        assert_eq!(
            verdict,
            RiskVerdict::Reject(risk_core::RejectReason::Margin)
        );
    }
}
