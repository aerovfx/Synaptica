use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project::Project;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct ProjectIdParam {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectBody {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub goal_id: Option<String>,
    pub lead_agent_id: Option<String>,
    pub target_date: Option<String>,
    pub color: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub goal_id: Option<String>,
    pub lead_agent_id: Option<String>,
    pub target_date: Option<String>,
    pub color: Option<String>,
}

/// GET /api/companies/:companyId/projects
pub async fn list_projects(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Project>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, Project>(
        "SELECT id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, execution_workspace_policy, archived_at, created_at, updated_at FROM projects WHERE company_id = $1 ORDER BY created_at",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("GET /api/companies/:company_id/projects failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(rows))
}

/// GET /api/projects/:id
pub async fn get_project(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectIdParam>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Project>(
        "SELECT id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, execution_workspace_policy, archived_at, created_at, updated_at FROM projects WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/projects
pub async fn create_project(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateProjectBody>,
) -> Result<(StatusCode, Json<Project>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let status = body.status.as_deref().unwrap_or("backlog");
    let goal_id: Option<Uuid> = body.goal_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let lead_agent_id: Option<Uuid> = body.lead_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let target_date: Option<chrono::NaiveDate> = body
        .target_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let row = sqlx::query_as::<_, Project>(
        "INSERT INTO projects (id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10) RETURNING id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, execution_workspace_policy, archived_at, created_at, updated_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(goal_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(status)
    .bind(lead_agent_id)
    .bind(target_date)
    .bind(body.color.as_deref())
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// PATCH /api/projects/:id
pub async fn update_project(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectIdParam>,
    Json(body): Json<UpdateProjectBody>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let goal_id: Option<Uuid> = body.goal_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let lead_agent_id: Option<Uuid> = body.lead_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let target_date: Option<chrono::NaiveDate> = body
        .target_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let row = sqlx::query_as::<_, Project>(
        "UPDATE projects SET name = COALESCE($2, name), description = COALESCE($3, description), status = COALESCE($4, status), goal_id = COALESCE($5, goal_id), lead_agent_id = COALESCE($6, lead_agent_id), target_date = COALESCE($7, target_date), color = COALESCE($8, color), updated_at = $9 WHERE id = $1 RETURNING id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, execution_workspace_policy, archived_at, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(body.name.as_deref())
    .bind(body.description.as_deref())
    .bind(body.status.as_deref())
    .bind(goal_id)
    .bind(lead_agent_id)
    .bind(target_date)
    .bind(body.color.as_deref())
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/projects/:id
pub async fn delete_project(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(&params.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn projects_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
