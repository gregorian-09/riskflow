//! Latency report for the pretrade gate.

use std::{
    env,
    hint::black_box,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use risk_core::{
    CurrencyId, DataQuality, EquitySpec, Instrument, InstrumentId, MarketPrice, MarketSnapshot,
    Notional, Price, Qty, Timestamp,
};
use risk_pretrade::{EvaluateRequest, LimitTable, PretradeGate};

const DEFAULT_ITERATIONS: usize = 50_000;

fn main() {
    let iterations = parse_iterations().unwrap_or(DEFAULT_ITERATIONS);
    let steady = measure_steady_read(iterations);
    let contended = measure_contended_updates(iterations);

    println!("pretrade evaluate latency report");
    println!("iterations: {iterations}");
    print_stats("steady_read", &steady);
    print_stats("contended_updates", &contended);
}

fn parse_iterations() -> Option<usize> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--iterations" {
            return args.next()?.parse().ok();
        }
    }

    None
}

fn measure_steady_read(iterations: usize) -> Vec<Duration> {
    let instrument = equity();
    let gate = PretradeGate::new(limits(1_000));
    let market = market();

    measure(iterations, || {
        evaluate_once(&gate, instrument, &market);
    })
}

fn measure_contended_updates(iterations: usize) -> Vec<Duration> {
    let instrument = equity();
    let gate = Arc::new(PretradeGate::new(limits(1_000)));
    let market = market();
    let running = Arc::new(AtomicBool::new(true));

    let writer_gate = Arc::clone(&gate);
    let writer_running = Arc::clone(&running);
    let writer = thread::spawn(move || {
        let mut version = 0_i64;
        while writer_running.load(Ordering::Relaxed) {
            version += 1;
            writer_gate.update_limits(limits(1_000 + version % 2));
        }
    });

    let samples = measure(iterations, || {
        evaluate_once(&gate, instrument, &market);
    });

    running.store(false, Ordering::Relaxed);
    writer.join().expect("limit-update worker panicked");

    samples
}

fn measure(mut iterations: usize, mut evaluate: impl FnMut()) -> Vec<Duration> {
    if iterations == 0 {
        iterations = DEFAULT_ITERATIONS;
    }

    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        evaluate();
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    samples
}

fn evaluate_once(gate: &PretradeGate, instrument: Instrument, market: &MarketSnapshot) {
    let verdict = gate.evaluate(EvaluateRequest {
        instrument,
        qty: Qty::new(5),
        current_position: Qty::new(0),
        available_margin: Notional::new(1_000),
        order_price: Price::new(100),
        market,
        now: Timestamp(10),
    });
    black_box(verdict);
}

fn print_stats(name: &str, samples: &[Duration]) {
    let median = percentile(samples, 500);
    let p999 = percentile(samples, 999);

    println!("{name}.median_ns: {}", median.as_nanos());
    println!("{name}.p99_9_ns: {}", p999.as_nanos());
}

fn percentile(samples: &[Duration], per_mille: usize) -> Duration {
    let len = samples.len();
    debug_assert!(len > 0);
    let index = (len.saturating_mul(per_mille).saturating_add(999) / 1_000)
        .saturating_sub(1)
        .min(len - 1);

    samples[index]
}

fn limits(per_order: i64) -> LimitTable {
    let mut limits = LimitTable::new();
    limits.set_per_order_notional(InstrumentId(1), Notional::new(per_order));
    limits.set_aggregate_notional(Notional::new(10_000));
    limits.set_max_abs_position(InstrumentId(1), Qty::new(100));
    limits.set_fat_finger_band_bps(InstrumentId(1), 500);
    limits.set_initial_margin_per_unit(InstrumentId(1), Notional::new(10));
    limits
}

fn market() -> MarketSnapshot {
    let mut market = MarketSnapshot::new(10, 10, 10);
    market.insert_price(
        InstrumentId(1),
        MarketPrice::clean(Price::new(100), Timestamp(5)),
    );
    market.set_aggregate_notional(Notional::new(0), Timestamp(5), DataQuality::clean());
    market
}

const fn equity() -> Instrument {
    Instrument::Equity(EquitySpec {
        instrument_id: InstrumentId(1),
        settlement_currency: CurrencyId(840),
    })
}
