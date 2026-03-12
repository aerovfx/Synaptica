//! Simple Prometheus-style metrics: request count, errors, active runs.
//! Rendered at GET /api/metrics.

use axum::body::Body;
use axum::http::Response;
use std::sync::atomic::{AtomicU64, Ordering};

static HTTP_REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static HTTP_ERRORS_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Global gauge for active heartbeat runs (updated by runner).
/// Use MetricsGauge::guard() when starting a run.
pub struct MetricsGauge {
    inner: std::sync::Arc<AtomicU64>,
}

impl MetricsGauge {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment(&self) {
        self.inner.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement(&self) {
        self.inner.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.inner.load(Ordering::Relaxed)
    }

    /// Call when starting a run; drop the guard when the run ends.
    pub fn guard(self: std::sync::Arc<Self>) -> MetricsGaugeGuard {
        self.increment();
        MetricsGaugeGuard { gauge: self }
    }
}

impl Clone for MetricsGauge {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct MetricsGaugeGuard {
    gauge: std::sync::Arc<MetricsGauge>,
}

impl Drop for MetricsGaugeGuard {
    fn drop(&mut self) {
        self.gauge.decrement();
    }
}

pub fn record_request() {
    HTTP_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_error() {
    HTTP_ERRORS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Render Prometheus text format. Pass optional active_runs (from ApiState) if DB is up.
pub fn render_prometheus(active_runs: Option<u64>) -> String {
    let requests = HTTP_REQUESTS_TOTAL.load(Ordering::Relaxed);
    let errors = HTTP_ERRORS_TOTAL.load(Ordering::Relaxed);
    let mut out = String::new();
    out.push_str("# HELP paperclip_http_requests_total Total HTTP requests\n");
    out.push_str("# TYPE paperclip_http_requests_total counter\n");
    out.push_str(&format!("paperclip_http_requests_total {}", requests));
    out.push_str("\n");
    out.push_str("# HELP paperclip_http_errors_total Total HTTP 4xx/5xx responses\n");
    out.push_str("# TYPE paperclip_http_errors_total counter\n");
    out.push_str(&format!("paperclip_http_errors_total {}", errors));
    out.push_str("\n");
    if let Some(n) = active_runs {
        out.push_str("# HELP paperclip_runner_active_runs Current number of adapter runs in progress\n");
        out.push_str("# TYPE paperclip_runner_active_runs gauge\n");
        out.push_str(&format!("paperclip_runner_active_runs {}", n));
        out.push_str("\n");
    }
    out
}

/// GET /api/metrics handler — returns Prometheus exposition format.
/// active_runs: from state.metrics_active_runs.get() when DB is set, else None.
pub async fn metrics_handler(active_runs: Option<u64>) -> Response<Body> {
    let body = render_prometheus(active_runs);
    Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from(body))
        .unwrap()
}

/// Middleware: record every request and count 4xx/5xx as errors.
pub async fn metrics_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    record_request();
    let response = next.run(request).await;
    if response.status().as_u16() >= 400 {
        record_error();
    }
    response
}
