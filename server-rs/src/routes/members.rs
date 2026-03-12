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
pub struct CompanyMembershipRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub principal_type: String,
    pub principal_id: String,
    pub status: String,
    pub membership_role: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

/// GET /api/companies/:company_id/members
pub async fn list_members(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CompanyMembershipRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, CompanyMembershipRow>(
        "SELECT id, company_id, principal_type, principal_id, status, membership_role, created_at, updated_at FROM company_memberships WHERE company_id = $1 ORDER BY created_at",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

pub async fn members_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
