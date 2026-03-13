use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::company_department::CompanyDepartment;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct CompanyDepartmentIdParam {
    pub company_id: String,
    pub department_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyDepartmentBody {
    pub name: String,
    pub space_id: Option<String>,
    pub leader_agent_id: Option<String>,
    pub order: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyDepartmentBody {
    pub name: Option<String>,
    pub space_id: Option<String>,
    pub leader_agent_id: Option<String>,
    pub order: Option<i32>,
}

/// GET /api/companies/:companyId/departments
pub async fn list_company_departments(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CompanyDepartment>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, CompanyDepartment>(
        "SELECT id, company_id, space_id, name, leader_agent_id, \"order\", created_at, updated_at FROM company_departments WHERE company_id = $1 ORDER BY \"order\", created_at",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/departments
pub async fn create_company_department(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateCompanyDepartmentBody>,
) -> Result<(StatusCode, Json<CompanyDepartment>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let space_id: Option<Uuid> = body.space_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let leader_agent_id: Option<Uuid> =
        body.leader_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let order = body.order.unwrap_or(0);
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanyDepartment>(
        "INSERT INTO company_departments (id, company_id, space_id, name, leader_agent_id, \"order\", created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $7) RETURNING id, company_id, space_id, name, leader_agent_id, \"order\", created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(space_id)
    .bind(&body.name)
    .bind(leader_agent_id)
    .bind(order)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/companies/:companyId/departments/:departmentId
pub async fn get_company_department(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyDepartmentIdParam>,
) -> Result<Json<CompanyDepartment>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let department_id = Uuid::parse_str(&params.department_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid department id".to_string()))?;
    let row = sqlx::query_as::<_, CompanyDepartment>(
        "SELECT id, company_id, space_id, name, leader_agent_id, \"order\", created_at, updated_at FROM company_departments WHERE id = $1 AND company_id = $2",
    )
    .bind(department_id)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Department not found".to_string()))?;
    Ok(Json(row))
}

/// PATCH /api/companies/:companyId/departments/:departmentId
pub async fn update_company_department(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyDepartmentIdParam>,
    Json(body): Json<UpdateCompanyDepartmentBody>,
) -> Result<Json<CompanyDepartment>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let department_id = Uuid::parse_str(&params.department_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid department id".to_string()))?;
    let space_id: Option<Uuid> = body.space_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let leader_agent_id: Option<Uuid> =
        body.leader_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanyDepartment>(
        "UPDATE company_departments SET name = COALESCE($2, name), space_id = COALESCE($3, space_id), leader_agent_id = COALESCE($4, leader_agent_id), \"order\" = COALESCE($5, \"order\"), updated_at = $6 WHERE id = $1 AND company_id = $7 RETURNING id, company_id, space_id, name, leader_agent_id, \"order\", created_at, updated_at",
    )
    .bind(department_id)
    .bind(&body.name)
    .bind(space_id)
    .bind(leader_agent_id)
    .bind(body.order)
    .bind(now)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Department not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId/departments/:departmentId
pub async fn delete_company_department(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyDepartmentIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let department_id = Uuid::parse_str(&params.department_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid department id".to_string()))?;

    let result = sqlx::query("DELETE FROM company_departments WHERE id = $1 AND company_id = $2")
        .bind(department_id)
        .bind(company_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Department not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn company_departments_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
