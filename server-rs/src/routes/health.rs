use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_exposure: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_ready: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_invite_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HealthFeatures>,
}

#[derive(Serialize)]
pub struct HealthFeatures {
    pub company_deletion_enabled: bool,
}

/// GET /api/health — contract matches Node server
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        deployment_mode: Some("local_trusted".to_string()),
        deployment_exposure: Some("private".to_string()),
        auth_ready: Some(true),
        bootstrap_status: Some("ready".to_string()),
        bootstrap_invite_active: Some(false),
        features: Some(HealthFeatures {
            company_deletion_enabled: true,
        }),
    })
}
