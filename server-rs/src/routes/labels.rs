//! Labels: list, create, delete (paperclip parity).

use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct LabelIdParam {
    pub label_id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLabelBody {
    pub name: String,
    pub color: String,
}

/// GET /api/companies/:companyId/labels
pub async fn list_labels(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Label>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, Label>(
        "SELECT id, company_id, name, color, created_at, updated_at FROM labels WHERE company_id = $1 ORDER BY name",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("GET /api/companies/:company_id/labels failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/labels
pub async fn create_label(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateLabelBody>,
) -> Result<(StatusCode, Json<Label>), (StatusCode, String)> {
    if body.name.is_empty() || body.name.len() > 48 {
        return Err((StatusCode::BAD_REQUEST, "name must be 1–48 characters".to_string()));
    }
    if !body.color.chars().all(|c| c.is_ascii_hexdigit() || c == '#') || body.color.len() != 7 || !body.color.starts_with('#') {
        return Err((StatusCode::BAD_REQUEST, "color must be #RRGGBB".to_string()));
    }
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Label>(
        "INSERT INTO labels (id, company_id, name, color, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5) RETURNING id, company_id, name, color, created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(body.name.trim())
    .bind(&body.color)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("POST /api/companies/:company_id/labels failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// DELETE /api/labels/:labelId
pub async fn delete_label(
    State(pool): State<PgPool>,
    Path(params): Path<LabelIdParam>,
) -> Result<Json<Label>, (StatusCode, String)> {
    let label_id = Uuid::parse_str(&params.label_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid label id".to_string()))?;
    let row = sqlx::query_as::<_, Label>(
        "DELETE FROM labels WHERE id = $1 RETURNING id, company_id, name, color, created_at, updated_at",
    )
    .bind(label_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!("DELETE /api/labels/:label_id failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Label not found".to_string()))?;
    Ok(Json(row))
}

pub async fn labels_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
