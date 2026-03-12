use axum::body::Bytes;
use axum::extract::Path;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::asset::Asset;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct AssetIdParam {
    pub id: String,
}

/// GET /api/companies/:company_id/assets
pub async fn list_assets(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Asset>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Asset>(
        "SELECT id, company_id, provider, object_key, content_type, byte_size, sha256, original_filename, created_by_agent_id, created_by_user_id, created_at, updated_at FROM assets WHERE company_id = $1 ORDER BY created_at DESC",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/assets/:id
pub async fn get_asset(
    State(pool): State<PgPool>,
    Path(params): Path<AssetIdParam>,
) -> Result<Json<Asset>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Asset>(
        "SELECT id, company_id, provider, object_key, content_type, byte_size, sha256, original_filename, created_by_agent_id, created_by_user_id, created_at, updated_at FROM assets WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Asset not found".to_string()))?;
    Ok(Json(row))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssetBody {
    pub content_base64: String,
    pub content_type: String,
    pub original_filename: Option<String>,
}

/// POST /api/companies/:company_id/assets — create asset (body: content_base64, content_type, original_filename)
pub async fn create_asset(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateAssetBody>,
) -> Result<(StatusCode, Json<Asset>), (StatusCode, String)> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &body.content_base64)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid base64 content".to_string()))?;
    let byte_size = bytes.len() as i32;
    let sha256_hex = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&bytes);
        format!("{:x}", h.finalize())
    };
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let provider = "local";
    let object_key = format!("{}/{}", params.company_id, id);
    if let Ok(base) = std::env::var("ASSETS_PATH") {
        let path = std::path::Path::new(&base).join(&object_key);
        if let Some(p) = path.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let _ = std::fs::write(&path, &bytes);
    }
    let row = sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (id, company_id, provider, object_key, content_type, byte_size, sha256, original_filename, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9) RETURNING id, company_id, provider, object_key, content_type, byte_size, sha256, original_filename, created_by_agent_id, created_by_user_id, created_at, updated_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(provider)
    .bind(&object_key)
    .bind(&body.content_type)
    .bind(byte_size)
    .bind(&sha256_hex)
    .bind(body.original_filename.as_deref())
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/assets/:id/content — serve file (requires ASSETS_PATH)
pub async fn get_asset_content(
    State(pool): State<PgPool>,
    Path(params): Path<AssetIdParam>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let row: (String, String) = sqlx::query_as("SELECT object_key, content_type FROM assets WHERE id = $1")
        .bind(&params.id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Asset not found".to_string()))?;
    let base = std::env::var("ASSETS_PATH").map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "ASSETS_PATH not set".to_string()))?;
    let path = std::path::Path::new(&base).join(&row.0);
    let bytes = std::fs::read(&path).map_err(|_| (StatusCode::NOT_FOUND, "Asset file not found".to_string()))?;
    Ok((
        [(header::CONTENT_TYPE, row.1)],
        Bytes::from(bytes),
    ))
}

/// DELETE /api/assets/:id
pub async fn delete_asset(
    State(pool): State<PgPool>,
    Path(params): Path<AssetIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let row: Option<(String, String)> = sqlx::query_as("SELECT company_id, object_key FROM assets WHERE id = $1")
        .bind(&params.id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if let Some((_, object_key)) = row {
        if let Ok(base) = std::env::var("ASSETS_PATH") {
            let path = std::path::Path::new(&base).join(object_key);
            let _ = std::fs::remove_file(path);
        }
    }
    let result = sqlx::query("DELETE FROM assets WHERE id = $1")
        .bind(&params.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Asset not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn assets_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
