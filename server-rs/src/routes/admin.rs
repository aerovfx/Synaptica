//! Admin and user company-access (instance admin scope).

use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct UserIdParam {
    pub user_id: String,
}

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

/// GET /api/admin/users/:userId/company-access
pub async fn get_user_company_access(
    State(pool): State<PgPool>,
    Path(params): Path<UserIdParam>,
) -> Result<Json<Vec<CompanyMembershipRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, CompanyMembershipRow>(
        "SELECT id, company_id, principal_type, principal_id, status, membership_role, created_at, updated_at FROM company_memberships WHERE principal_type = 'user' AND principal_id = $1 ORDER BY created_at DESC",
    )
    .bind(&params.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetUserCompanyAccessBody {
    pub company_ids: Option<Vec<String>>,
}

/// PUT /api/admin/users/:userId/company-access
pub async fn put_user_company_access(
    State(pool): State<PgPool>,
    Path(params): Path<UserIdParam>,
    Json(body): Json<SetUserCompanyAccessBody>,
) -> Result<Json<Vec<CompanyMembershipRow>>, (StatusCode, String)> {
    let user_id = &params.user_id;
    let company_ids = body.company_ids.as_deref().unwrap_or(&[]);
    let existing: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, company_id FROM company_memberships WHERE principal_type = 'user' AND principal_id = $1",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let target: Vec<Uuid> = company_ids
        .iter()
        .filter_map(|s| Uuid::parse_str(s).ok())
        .collect();
    let existing_cids: Vec<Uuid> = existing.iter().map(|(_, cid)| *cid).collect();
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    for (id, cid) in &existing {
        if !target.contains(cid) {
            sqlx::query("DELETE FROM company_memberships WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }
    for cid in &target {
        if existing_cids.contains(cid) {
            continue;
        }
        let now = chrono::Utc::now();
        sqlx::query(
            "INSERT INTO company_memberships (id, company_id, principal_type, principal_id, status, membership_role, updated_at) VALUES (gen_random_uuid(), $1, 'user', $2, 'active', 'member', $3)",
        )
        .bind(cid)
        .bind(user_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rows = sqlx::query_as::<_, CompanyMembershipRow>(
        "SELECT id, company_id, principal_type, principal_id, status, membership_role, created_at, updated_at FROM company_memberships WHERE principal_type = 'user' AND principal_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InstanceAdminRow {
    pub id: Uuid,
    pub user_id: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/admin/users/:userId/promote-instance-admin
pub async fn promote_instance_admin(
    State(pool): State<PgPool>,
    Path(params): Path<UserIdParam>,
) -> Result<(StatusCode, Json<InstanceAdminRow>), (StatusCode, String)> {
    let existing = sqlx::query_as::<_, InstanceAdminRow>(
        "SELECT id, user_id, role, created_at, updated_at FROM instance_user_roles WHERE user_id = $1 AND role = 'instance_admin'",
    )
    .bind(&params.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some(row) = existing {
        return Ok((StatusCode::OK, Json(row)));
    }
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO instance_user_roles (id, user_id, role, created_at, updated_at) VALUES ($1, $2, 'instance_admin', $3, $3)",
    )
    .bind(id)
    .bind(&params.user_id)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let row = sqlx::query_as::<_, InstanceAdminRow>(
        "SELECT id, user_id, role, created_at, updated_at FROM instance_user_roles WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// POST /api/admin/users/:userId/demote-instance-admin
pub async fn demote_instance_admin(
    State(pool): State<PgPool>,
    Path(params): Path<UserIdParam>,
) -> Result<Json<InstanceAdminRow>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, InstanceAdminRow>(
        "DELETE FROM instance_user_roles WHERE user_id = $1 AND role = 'instance_admin' RETURNING id, user_id, role, created_at, updated_at",
    )
    .bind(&params.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Instance admin role not found".to_string()))?;
    Ok(Json(row))
}

pub async fn admin_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
