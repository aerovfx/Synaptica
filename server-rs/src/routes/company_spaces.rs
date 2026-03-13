use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::company_space::CompanySpace;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct CompanySpaceIdParam {
    pub company_id: String,
    pub space_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanySpaceBody {
    pub name: String,
    pub parent_id: Option<String>,
    pub order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanySpaceBody {
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub order: Option<i32>,
}

/// GET /api/companies/:companyId/spaces
pub async fn list_company_spaces(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CompanySpace>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, CompanySpace>(
        "SELECT id, company_id, parent_id, name, \"order\", created_at, updated_at FROM company_spaces WHERE company_id = $1 ORDER BY \"order\", created_at",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/spaces
pub async fn create_company_space(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateCompanySpaceBody>,
) -> Result<(StatusCode, Json<CompanySpace>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let parent_id: Option<Uuid> = body.parent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let order = body.order.unwrap_or(0);
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanySpace>(
        "INSERT INTO company_spaces (id, company_id, parent_id, name, \"order\", created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6) RETURNING id, company_id, parent_id, name, \"order\", created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(parent_id)
    .bind(&body.name)
    .bind(order)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/companies/:companyId/spaces/:spaceId
pub async fn get_company_space(
    State(pool): State<PgPool>,
    Path(params): Path<CompanySpaceIdParam>,
) -> Result<Json<CompanySpace>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let space_id = Uuid::parse_str(&params.space_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid space id".to_string()))?;
    let row = sqlx::query_as::<_, CompanySpace>(
        "SELECT id, company_id, parent_id, name, \"order\", created_at, updated_at FROM company_spaces WHERE id = $1 AND company_id = $2",
    )
    .bind(space_id)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Space not found".to_string()))?;
    Ok(Json(row))
}

/// PATCH /api/companies/:companyId/spaces/:spaceId
pub async fn update_company_space(
    State(pool): State<PgPool>,
    Path(params): Path<CompanySpaceIdParam>,
    Json(body): Json<UpdateCompanySpaceBody>,
) -> Result<Json<CompanySpace>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let space_id = Uuid::parse_str(&params.space_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid space id".to_string()))?;
    let parent_id: Option<Uuid> = body.parent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanySpace>(
        "UPDATE company_spaces SET name = COALESCE($2, name), parent_id = COALESCE($3, parent_id), \"order\" = COALESCE($4, \"order\"), updated_at = $5 WHERE id = $1 AND company_id = $6 RETURNING id, company_id, parent_id, name, \"order\", created_at, updated_at",
    )
    .bind(space_id)
    .bind(&body.name)
    .bind(parent_id)
    .bind(body.order)
    .bind(now)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Space not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId/spaces/:spaceId
pub async fn delete_company_space(
    State(pool): State<PgPool>,
    Path(params): Path<CompanySpaceIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let space_id = Uuid::parse_str(&params.space_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid space id".to_string()))?;

    let result = sqlx::query("DELETE FROM company_spaces WHERE id = $1 AND company_id = $2")
        .bind(space_id)
        .bind(company_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Space not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn company_spaces_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
