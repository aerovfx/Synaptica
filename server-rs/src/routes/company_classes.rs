use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::company_class::CompanyClass;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct CompanyClassIdParam {
    pub company_id: String,
    pub class_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyClassBody {
    pub name: String,
    pub description: Option<String>,
    pub order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyClassBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub order: Option<i32>,
}

/// GET /api/companies/:companyId/classes
pub async fn list_company_classes(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CompanyClass>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, CompanyClass>(
        "SELECT id, company_id, name, description, \"order\", created_at, updated_at FROM company_classes WHERE company_id = $1 ORDER BY \"order\", created_at",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/classes
pub async fn create_company_class(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateCompanyClassBody>,
) -> Result<(StatusCode, Json<CompanyClass>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let order = body.order.unwrap_or(0);
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanyClass>(
        "INSERT INTO company_classes (id, company_id, name, description, \"order\", created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6) RETURNING id, company_id, name, description, \"order\", created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(order)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/companies/:companyId/classes/:classId
pub async fn get_company_class(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyClassIdParam>,
) -> Result<Json<CompanyClass>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let class_id = Uuid::parse_str(&params.class_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid class id".to_string()))?;
    let row = sqlx::query_as::<_, CompanyClass>(
        "SELECT id, company_id, name, description, \"order\", created_at, updated_at FROM company_classes WHERE id = $1 AND company_id = $2",
    )
    .bind(class_id)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Class not found".to_string()))?;
    Ok(Json(row))
}

/// PATCH /api/companies/:companyId/classes/:classId
pub async fn update_company_class(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyClassIdParam>,
    Json(body): Json<UpdateCompanyClassBody>,
) -> Result<Json<CompanyClass>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let class_id = Uuid::parse_str(&params.class_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid class id".to_string()))?;
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanyClass>(
        "UPDATE company_classes SET name = COALESCE($2, name), description = COALESCE($3, description), \"order\" = COALESCE($4, \"order\"), updated_at = $5 WHERE id = $1 AND company_id = $6 RETURNING id, company_id, name, description, \"order\", created_at, updated_at",
    )
    .bind(class_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.order)
    .bind(now)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Class not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId/classes/:classId
pub async fn delete_company_class(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyClassIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let class_id = Uuid::parse_str(&params.class_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid class id".to_string()))?;

    let result = sqlx::query("DELETE FROM company_classes WHERE id = $1 AND company_id = $2")
        .bind(class_id)
        .bind(company_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Class not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn company_classes_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
