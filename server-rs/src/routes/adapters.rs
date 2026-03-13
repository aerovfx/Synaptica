//! Stub routes for adapter models and test-environment when running Rust-only server.
//! Full implementation lives in Node/CLI; these prevent 404/405 and return empty or stub responses.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

/// GET /api/companies/:company_id/adapters/:adapter_type/models — stub: empty list (Rust server has no adapter runtime).
pub async fn get_adapter_models(
    Path((_company_id, _adapter_type)): Path<(String, String)>,
) -> Json<Vec<serde_json::Value>> {
    Json(vec![])
}

/// POST /api/companies/:company_id/adapters/:adapter_type/test-environment — stub: warn that test is not available.
pub async fn post_test_environment(
    Path((_company_id, adapter_type)): Path<(String, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let body = json!({
        "adapterType": adapter_type,
        "status": "warn",
        "checks": [{
            "name": "server",
            "status": "warn",
            "message": "Environment test is not available in this server (Rust-only build). Use CLI or Node server for full adapter support."
        }],
        "testedAt": chrono::Utc::now().to_rfc3339()
    });
    (StatusCode::OK, Json(body))
}
