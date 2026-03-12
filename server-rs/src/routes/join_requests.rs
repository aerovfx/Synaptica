use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct JoinRequestRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub invite_id: Uuid,
    pub request_type: String,
    pub status: String,
    pub agent_name: Option<String>,
    pub adapter_type: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct JoinRequestIdParam {
    pub id: String,
}

/// GET /api/companies/:company_id/join-requests
pub async fn list_join_requests(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<JoinRequestRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, JoinRequestRow>(
        "SELECT id, company_id, invite_id, request_type, status, agent_name, adapter_type, created_at, updated_at FROM join_requests WHERE company_id = $1 ORDER BY created_at DESC",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/join-requests/:id
pub async fn get_join_request(
    State(pool): State<PgPool>,
    Path(params): Path<JoinRequestIdParam>,
) -> Result<Json<JoinRequestRow>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, JoinRequestRow>(
        "SELECT id, company_id, invite_id, request_type, status, agent_name, adapter_type, created_at, updated_at FROM join_requests WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Join request not found".to_string()))?;
    Ok(Json(row))
}

pub async fn join_requests_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
