use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub actor_type: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub agent_id: Option<uuid::Uuid>,
    pub run_id: Option<uuid::Uuid>,
    pub details: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /api/companies/:companyId/activity
pub async fn list_activity(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<ActivityEntry>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ActivityEntry>(
        "SELECT id, company_id, actor_type, actor_id, action, entity_type, entity_id, agent_id, run_id, details, created_at FROM activity_log WHERE company_id = $1 ORDER BY created_at DESC LIMIT 200",
    )
    .bind(params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

pub async fn activity_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set",
    )
}
