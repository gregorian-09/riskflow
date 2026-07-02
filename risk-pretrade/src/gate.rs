//! Pretrade gate and limit table.

use std::{collections::HashMap, sync::Arc};

use risk_core::{
    IndeterminateReason, Instrument, InstrumentId, MarketSnapshot, Notional, Qty, RejectReason,
    RiskVerdict, RiskWeight, Timestamp,
};

/// Immutable pretrade limit table.
#[derive(Debug, Clone, Default)]
pub struct LimitTable {
    per_order_notional: HashMap<InstrumentId, Notional>,
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

    /// Returns the per-order notional limit for an instrument.
    #[must_use]
    pub fn per_order_notional(&self, instrument_id: InstrumentId) -> Option<Notional> {
        self.per_order_notional.get(&instrument_id).copied()
    }
}

/// Request evaluated by the pretrade gate.
#[derive(Debug, Clone, Copy)]
pub struct EvaluateRequest<'a> {
    /// Static instrument reference data.
    pub instrument: Instrument,
    /// Order quantity.
    pub qty: Qty,
    /// Market snapshot.
    pub market: &'a MarketSnapshot,
    /// Current timestamp.
    pub now: Timestamp,
}

/// Synchronous pretrade risk gate.
#[derive(Debug, Clone)]
pub struct PretradeGate {
    limits: Arc<LimitTable>,
}

impl PretradeGate {
    /// Creates a gate from an immutable limit table snapshot.
    #[must_use]
    pub fn new(limits: LimitTable) -> Self {
        Self {
            limits: Arc::new(limits),
        }
    }

    /// Replaces the active limit table snapshot.
    ///
    /// This is intentionally isolated so the implementation can move to
    /// `ArcSwap<LimitTable>` without changing check logic.
    pub fn update_limits(&mut self, limits: LimitTable) {
        self.limits = Arc::new(limits);
    }

    /// Evaluates an order request.
    #[must_use]
    pub fn evaluate(&self, request: EvaluateRequest<'_>) -> RiskVerdict {
        match request
            .instrument
            .risk_weight(request.qty, request.market, request.now)
        {
            RiskWeight::Linear(notional) => {
                self.check_per_order_notional(request.instrument.id(), notional)
            }
            RiskWeight::Indeterminate(reason) => RiskVerdict::Indeterminate(reason),
        }
    }

    fn check_per_order_notional(
        &self,
        instrument_id: InstrumentId,
        notional: Notional,
    ) -> RiskVerdict {
        let Some(limit) = self.limits.per_order_notional(instrument_id) else {
            return RiskVerdict::Indeterminate(IndeterminateReason::MissingLimit);
        };

        if notional > limit {
            RiskVerdict::Reject(RejectReason::OrderNotionalLimit)
        } else {
            RiskVerdict::Pass
        }
    }
}

#[cfg(test)]
mod tests {
    use risk_core::{
        CurrencyId, EquitySpec, Instrument, InstrumentId, MarketPrice, MarketSnapshot, Notional,
        OptionSpec, Price, Qty, RiskVerdict, Timestamp,
    };

    use super::{EvaluateRequest, LimitTable, PretradeGate};

    #[test]
    fn option_order_is_indeterminate_without_risk_options_dependency() {
        let option = Instrument::Option(OptionSpec {
            instrument_id: InstrumentId(1),
            underlying_id: InstrumentId(2),
            settlement_currency: CurrencyId(840),
            expiry: Timestamp(1_000),
        });
        let mut limits = LimitTable::new();
        limits.set_per_order_notional(InstrumentId(1), Notional::new(1_000));
        let gate = PretradeGate::new(limits);
        let market = MarketSnapshot::new(10, 10, 10);

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: option,
            qty: Qty::new(1),
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
        let mut limits = LimitTable::new();
        limits.set_per_order_notional(InstrumentId(1), Notional::new(1_000));
        let gate = PretradeGate::new(limits);
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(100), Timestamp(5)),
        );

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            market: &market,
            now: Timestamp(10),
        });

        assert_eq!(verdict, RiskVerdict::Pass);
    }

    #[test]
    fn linear_order_rejects_above_limit() {
        let equity = Instrument::Equity(EquitySpec {
            instrument_id: InstrumentId(1),
            settlement_currency: CurrencyId(840),
        });
        let mut limits = LimitTable::new();
        limits.set_per_order_notional(InstrumentId(1), Notional::new(100));
        let gate = PretradeGate::new(limits);
        let mut market = MarketSnapshot::new(10, 10, 10);
        market.insert_price(
            InstrumentId(1),
            MarketPrice::clean(Price::new(100), Timestamp(5)),
        );

        let verdict = gate.evaluate(EvaluateRequest {
            instrument: equity,
            qty: Qty::new(5),
            market: &market,
            now: Timestamp(10),
        });

        assert!(matches!(verdict, RiskVerdict::Reject(_)));
    }
}
