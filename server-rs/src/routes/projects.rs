use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project::Project;

/// Row shape when execution_workspace_policy column is missing (pre-0027).
#[derive(Debug, sqlx::FromRow)]
struct ProjectRowNoEwp {
    id: Uuid,
    company_id: Uuid,
    goal_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    status: String,
    lead_agent_id: Option<Uuid>,
    target_date: Option<chrono::NaiveDate>,
    color: Option<String>,
    archived_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Row shape when execution_workspace_policy, archived_at and color are missing (base schema).
#[derive(Debug, sqlx::FromRow)]
struct ProjectRowMinimal {
    id: Uuid,
    company_id: Uuid,
    goal_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    status: String,
    lead_agent_id: Option<Uuid>,
    target_date: Option<chrono::NaiveDate>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

fn project_from_no_ewp(r: ProjectRowNoEwp) -> Project {
    Project {
        id: r.id,
        company_id: r.company_id,
        goal_id: r.goal_id,
        name: r.name,
        description: r.description,
        status: r.status,
        lead_agent_id: r.lead_agent_id,
        target_date: r.target_date,
        color: r.color,
        execution_workspace_policy: None,
        archived_at: r.archived_at,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

fn project_from_minimal(r: ProjectRowMinimal) -> Project {
    Project {
        id: r.id,
        company_id: r.company_id,
        goal_id: r.goal_id,
        name: r.name,
        description: r.description,
        status: r.status,
        lead_agent_id: r.lead_agent_id,
        target_date: r.target_date,
        color: None,
        execution_workspace_policy: None,
        archived_at: None,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

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
    let full_query = "SELECT id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, execution_workspace_policy, archived_at, created_at, updated_at FROM projects WHERE company_id = $1 ORDER BY created_at";
    match sqlx::query_as::<_, Project>(full_query)
        .bind(company_id)
        .fetch_all(&pool)
        .await
    {
        Ok(rows) => Ok(Json(rows)),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("does not exist") || err_str.contains("relation") || err_str.contains("column") {
                // If projects table/columns are missing (old schema), treat as no projects instead of 500.
                tracing::warn!(
                    "GET /api/companies/:company_id/projects schema issue (treating as empty list). Error: {}",
                    e
                );
                Ok(Json(Vec::new()))
            } else {
                tracing::error!("GET /api/companies/:company_id/projects failed: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, err_str))
            }
        }
    }
}

const PROJECT_SELECT_FULL: &str = "SELECT id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, execution_workspace_policy, archived_at, created_at, updated_at FROM projects WHERE id = $1";
const PROJECT_SELECT_NO_EWP: &str = "SELECT id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, archived_at, created_at, updated_at FROM projects WHERE id = $1";
const PROJECT_SELECT_MINIMAL: &str = "SELECT id, company_id, goal_id, name, description, status, lead_agent_id, target_date, created_at, updated_at FROM projects WHERE id = $1";

/// GET /api/projects/:id
pub async fn get_project(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectIdParam>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let project_id = Uuid::parse_str(&params.id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid project id".to_string()))?;

    if let Ok(Some(row)) = sqlx::query_as::<_, Project>(PROJECT_SELECT_FULL)
        .bind(project_id)
        .fetch_optional(&pool)
        .await
    {
        return Ok(Json(row));
    }

    if let Ok(Some(row)) = sqlx::query_as::<_, ProjectRowNoEwp>(PROJECT_SELECT_NO_EWP)
        .bind(project_id)
        .fetch_optional(&pool)
        .await
    {
        return Ok(Json(project_from_no_ewp(row)));
    }

    if let Ok(Some(row)) = sqlx::query_as::<_, ProjectRowMinimal>(PROJECT_SELECT_MINIMAL)
        .bind(project_id)
        .fetch_optional(&pool)
        .await
    {
        return Ok(Json(project_from_minimal(row)));
    }

    let exists = sqlx::query_scalar::<_, i32>("SELECT 1 FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(&pool)
        .await
        .map(|o| o == Some(1))
        .unwrap_or(false);
    if !exists {
        return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
    }
    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Project exists but could not be loaded; check schema (e.g. run pnpm db:migrate)".to_string(),
    ))
}

/// POST /api/companies/:companyId/projects
pub async fn create_project(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateProjectBody>,
) -> Result<(StatusCode, Json<Project>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let now = chrono::Utc::now();
    let status = body.status.as_deref().unwrap_or("backlog");
    let goal_id: Option<Uuid> = body.goal_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let lead_agent_id: Option<Uuid> =
        body.lead_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let target_date: Option<chrono::NaiveDate> = body
        .target_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let insert_only = "INSERT INTO projects (id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10)";
    let full_returning = "INSERT INTO projects (id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10) RETURNING id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, execution_workspace_policy, archived_at, created_at, updated_at";

    let row = match sqlx::query_as::<_, Project>(full_returning)
        .bind(id)
        .bind(company_id)
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
    {
        Ok(r) => r,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("does not exist") || err_str.contains("column") {
                tracing::warn!(
                    "POST /api/companies/:company_id/projects RETURNING failed (schema?), using insert + fallback select: {}",
                    e
                );
                sqlx::query(insert_only)
                    .bind(id)
                    .bind(company_id)
                    .bind(goal_id)
                    .bind(&body.name)
                    .bind(&body.description)
                    .bind(status)
                    .bind(lead_agent_id)
                    .bind(target_date)
                    .bind(body.color.as_deref())
                    .bind(now)
                    .execute(&pool)
                    .await
                    .map_err(|e2| {
                        tracing::error!(
                            "POST /api/companies/:company_id/projects INSERT failed: {}",
                            e2
                        );
                        (StatusCode::INTERNAL_SERVER_ERROR, e2.to_string())
                    })?;
                // If we got here, table exists but columns are out of date; surface a clear hint.
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "Database schema issue creating project. Try running: pnpm db:migrate. Underlying error: {}",
                        err_str
                    ),
                ));
            } else {
                tracing::error!("POST /api/companies/:company_id/projects failed: {}", e);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, err_str));
            }
        }
    };

    Ok((StatusCode::CREATED, Json(row)))
}

/// PATCH /api/projects/:id
pub async fn update_project(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectIdParam>,
    Json(body): Json<UpdateProjectBody>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let project_id = Uuid::parse_str(&params.id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid project id".to_string()))?;
    let now = chrono::Utc::now();
    let goal_id: Option<Uuid> = body.goal_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let lead_agent_id: Option<Uuid> = body.lead_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let target_date: Option<chrono::NaiveDate> = body
        .target_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let update_sql = "UPDATE projects SET name = COALESCE($2, name), description = COALESCE($3, description), status = COALESCE($4, status), goal_id = COALESCE($5, goal_id), lead_agent_id = COALESCE($6, lead_agent_id), target_date = COALESCE($7, target_date), color = COALESCE($8, color), updated_at = $9 WHERE id = $1";
    let updated = sqlx::query(update_sql)
        .bind(project_id)
        .bind(body.name.as_deref())
        .bind(body.description.as_deref())
        .bind(body.status.as_deref())
        .bind(goal_id)
        .bind(lead_agent_id)
        .bind(target_date)
        .bind(body.color.as_deref())
        .bind(now)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if updated.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Project not found".to_string()));
    }

    if let Ok(Some(row)) = sqlx::query_as::<_, Project>(PROJECT_SELECT_FULL)
        .bind(project_id)
        .fetch_optional(&pool)
        .await
    {
        return Ok(Json(row));
    }
    if let Ok(Some(row)) = sqlx::query_as::<_, ProjectRowNoEwp>(PROJECT_SELECT_NO_EWP)
        .bind(project_id)
        .fetch_optional(&pool)
        .await
    {
        return Ok(Json(project_from_no_ewp(row)));
    }
    if let Ok(Some(row)) = sqlx::query_as::<_, ProjectRowMinimal>(PROJECT_SELECT_MINIMAL)
        .bind(project_id)
        .fetch_optional(&pool)
        .await
    {
        return Ok(Json(project_from_minimal(row)));
    }
    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Project updated but could not load result; check schema (e.g. run pnpm db:migrate)".to_string(),
    ))
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
