use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::company_secret::CompanySecret;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct SecretIdParam {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretBody {
    pub name: String,
    pub description: Option<String>,
    pub provider: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecretBody {
    pub description: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RotateSecretBody {
    pub material: Option<serde_json::Value>,
}

/// GET /api/companies/:companyId/secret-providers — list providers (stub: empty array for Rust).
pub async fn list_secret_providers(
    State(_pool): State<PgPool>,
    Path(_params): Path<CompanyIdParam>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    Ok(Json(Vec::new()))
}

/// GET /api/companies/:company_id/secrets
pub async fn list_secrets(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CompanySecret>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, CompanySecret>(
        "SELECT id, company_id, name, provider, external_ref, latest_version, description, created_by_agent_id, created_by_user_id, created_at, updated_at FROM company_secrets WHERE company_id = $1 ORDER BY name",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/secrets/:id
pub async fn get_secret(
    State(pool): State<PgPool>,
    Path(params): Path<SecretIdParam>,
) -> Result<Json<CompanySecret>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, CompanySecret>(
        "SELECT id, company_id, name, provider, external_ref, latest_version, description, created_by_agent_id, created_by_user_id, created_at, updated_at FROM company_secrets WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Secret not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:company_id/secrets
pub async fn create_secret(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateSecretBody>,
) -> Result<(StatusCode, Json<CompanySecret>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let provider = body.provider.as_deref().unwrap_or("local_encrypted");
    let row = sqlx::query_as::<_, CompanySecret>(
        "INSERT INTO company_secrets (id, company_id, name, provider, latest_version, description, created_at, updated_at) VALUES ($1, $2, $3, $4, 1, $5, $6, $6) RETURNING id, company_id, name, provider, external_ref, latest_version, description, created_by_agent_id, created_by_user_id, created_at, updated_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(&body.name)
    .bind(provider)
    .bind(&body.description)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "INSERT INTO company_secret_versions (id, secret_id, version, material, value_sha256, created_at) VALUES ($1, $2, 1, '{}', '', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// PATCH /api/secrets/:id
pub async fn update_secret(
    State(pool): State<PgPool>,
    Path(params): Path<SecretIdParam>,
    Json(body): Json<UpdateSecretBody>,
) -> Result<Json<CompanySecret>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, CompanySecret>(
        "UPDATE company_secrets SET description = COALESCE($2, description), updated_at = $3 WHERE id = $1 RETURNING id, company_id, name, provider, external_ref, latest_version, description, created_by_agent_id, created_by_user_id, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(body.description.as_deref())
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Secret not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/secrets/:id/rotate — create new version
pub async fn rotate_secret(
    State(pool): State<PgPool>,
    Path(params): Path<SecretIdParam>,
    Json(body): Json<RotateSecretBody>,
) -> Result<Json<CompanySecret>, (StatusCode, String)> {
    let secret_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid secret id".to_string()))?;
    let (version,): (i32,) = sqlx::query_as(
        "SELECT latest_version FROM company_secrets WHERE id = $1",
    )
    .bind(secret_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Secret not found".to_string()))?;
    let new_version = version + 1;
    let material = body.material.unwrap_or(serde_json::json!({}));
    let value_sha256 = ""; // placeholder when not encrypting
    let now = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO company_secret_versions (id, secret_id, version, material, value_sha256, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(secret_id)
    .bind(new_version)
    .bind(&material)
    .bind(value_sha256)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "UPDATE company_secrets SET latest_version = $2, updated_at = $3 WHERE id = $1",
    )
    .bind(secret_id)
    .bind(new_version)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let row = sqlx::query_as::<_, CompanySecret>(
        "SELECT id, company_id, name, provider, external_ref, latest_version, description, created_by_agent_id, created_by_user_id, created_at, updated_at FROM company_secrets WHERE id = $1",
    )
    .bind(secret_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/secrets/:id
pub async fn delete_secret(
    State(pool): State<PgPool>,
    Path(params): Path<SecretIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM company_secrets WHERE id = $1")
        .bind(&params.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Secret not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn secrets_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
