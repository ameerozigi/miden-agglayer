//! Lightweight metrics helper for the miden-agglayer crate.
//!
//! This module provides a tiny, dependency-free metrics store using atomic counters
//! and gauges. It exposes a Prometheus-compatible text format via `render_prometheus`.
//!
//! It also contains a small `metrics_handler` that can be mounted into an `axum` router
//! to expose `/metrics` for scraping. `axum` is already a dependency of the workspace
//! so the handler uses a simple tuple response.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct AggregationMetrics {
    start: Instant,
}

impl AggregationMetrics {
    /// Start a new aggregation timing window
    pub fn start() -> Self {
        Self { start: Instant::now() }
    }

    /// Finish timing and return elapsed duration
    pub fn finish(self) -> Duration {
        self.start.elapsed()
    }
}

/// Map of counter name -> value
static COUNTERS: OnceLock<Mutex<HashMap<String, Arc<AtomicU64>>>> = OnceLock::new();

/// Map of gauge name -> value
static GAUGES: OnceLock<Mutex<HashMap<String, Arc<AtomicU64>>>> = OnceLock::new();

fn counters_map() -> &'static Mutex<HashMap<String, Arc<AtomicU64>>> {
    COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn gauges_map() -> &'static Mutex<HashMap<String, Arc<AtomicU64>>> {
    GAUGES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Increment a named counter by `delta` (create if missing).
pub fn inc_counter(name: &str, delta: u64) {
    let map = counters_map();
    let entry = {
        let mut guard = map.lock().expect("counter map lock");
        guard
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone()
    };
    entry.fetch_add(delta, Ordering::Relaxed);
}

/// Set a gauge to the provided integer value (create if missing).
pub fn set_gauge(name: &str, value: u64) {
    let map = gauges_map();
    let entry = {
        let mut guard = map.lock().expect("gauge map lock");
        guard
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone()
    };
    entry.store(value, Ordering::Relaxed);
}

/// Render the current metrics in Prometheus text exposition format.
///
/// Example output:
///
/// # TYPE my_counter counter
/// my_counter 42
/// # TYPE my_gauge gauge
/// my_gauge 7
pub fn render_prometheus() -> String {
    let mut out = String::new();

    // Render counters
    if let Ok(guard) = counters_map().lock() {
        for (name, val) in guard.iter() {
            // Prometheus metric names should be ascii and follow rules; we keep names as-is here.
            let _ = writeln!(&mut out, "# TYPE {} counter", name);
            let _ = writeln!(&mut out, "{} {}", name, val.load(Ordering::Relaxed));
        }
    }

    // Render gauges
    if let Ok(guard) = gauges_map().lock() {
        for (name, val) in guard.iter() {
            let _ = writeln!(&mut out, "# TYPE {} gauge", name);
            let _ = writeln!(&mut out, "{} {}", name, val.load(Ordering::Relaxed));
        }
    }

    out
}

use std::fmt::Write;

/// Axum handler exposing the metrics in text format suitable for Prometheus scraping.
///
/// Mount it like:
///
///     .route("/metrics", get(metrics::metrics_handler))
///
pub async fn metrics_handler() -> (axum::http::StatusCode, axum::http::HeaderMap, String) {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    let body = render_prometheus();
    (axum::http::StatusCode::OK, headers, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_counter_and_gauge() {
        inc_counter("test_counter", 1);
        inc_counter("test_counter", 2);
        set_gauge("test_gauge", 7);

        let out = render_prometheus();
        assert!(out.contains("test_counter 3"));
        assert!(out.contains("test_gauge 7"));
    }
}
