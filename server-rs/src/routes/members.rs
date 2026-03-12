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

#[derive(Deserialize)]
pub struct CompanyAndMemberIdParam {
    pub company_id: String,
    pub member_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantInput {
    pub permission_key: String,
    pub scope: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberPermissionsBody {
    pub grants: Option<Vec<GrantInput>>,
}

/// PATCH /api/companies/:companyId/members/:memberId/permissions
pub async fn update_member_permissions(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyAndMemberIdParam>,
    Json(body): Json<UpdateMemberPermissionsBody>,
) -> Result<Json<CompanyMembershipRow>, (StatusCode, String)> {
    let member_id = Uuid::parse_str(&params.member_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid member id".to_string()))?;
    let company_id = Uuid::parse_str(&params.company_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let member: (String, String) = sqlx::query_as(
        "SELECT principal_type, principal_id FROM company_memberships WHERE id = $1 AND company_id = $2",
    )
    .bind(member_id)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Member not found".to_string()))?;
    let (principal_type, principal_id) = member;
    let grants = body.grants.as_deref().unwrap_or(&[]);
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "DELETE FROM principal_permission_grants WHERE company_id = $1 AND principal_type = $2 AND principal_id = $3",
    )
    .bind(company_id)
    .bind(&principal_type)
    .bind(&principal_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    for g in grants {
        let grant_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        sqlx::query(
            "INSERT INTO principal_permission_grants (id, company_id, principal_type, principal_id, permission_key, scope, granted_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
        )
        .bind(grant_id)
        .bind(company_id)
        .bind(&principal_type)
        .bind(&principal_id)
        .bind(&g.permission_key)
        .bind(g.scope.as_ref())
        .bind::<Option<String>>(None)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let row = sqlx::query_as::<_, CompanyMembershipRow>(
        "SELECT id, company_id, principal_type, principal_id, status, membership_role, created_at, updated_at FROM company_memberships WHERE id = $1",
    )
    .bind(member_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Member not found".to_string()))?;
    Ok(Json(row))
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
