use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::project_workspace::ProjectWorkspace;

#[derive(Deserialize)]
pub struct ProjectIdParam {
    pub id: String,
}

#[derive(Deserialize)]
pub struct ProjectWorkspaceIdParam {
    pub id: String,
    pub workspace_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceBody {
    pub name: String,
    pub cwd: Option<String>,
    pub repo_url: Option<String>,
    pub repo_ref: Option<String>,
    pub is_primary: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceBody {
    pub name: Option<String>,
    pub cwd: Option<String>,
    pub repo_url: Option<String>,
    pub repo_ref: Option<String>,
    pub is_primary: Option<bool>,
}

/// GET /api/projects/:id/workspaces
pub async fn list_workspaces(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectIdParam>,
) -> Result<Json<Vec<ProjectWorkspace>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ProjectWorkspace>(
        "SELECT id, company_id, project_id, name, cwd, repo_url, repo_ref, metadata, is_primary, created_at, updated_at FROM project_workspaces WHERE project_id = $1 ORDER BY created_at",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/projects/:id/workspaces/:workspace_id
pub async fn get_workspace(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectWorkspaceIdParam>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, ProjectWorkspace>(
        "SELECT id, company_id, project_id, name, cwd, repo_url, repo_ref, metadata, is_primary, created_at, updated_at FROM project_workspaces WHERE id = $1 AND project_id = $2",
    )
    .bind(&params.workspace_id)
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Workspace not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/projects/:id/workspaces
pub async fn create_workspace(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectIdParam>,
    Json(body): Json<CreateWorkspaceBody>,
) -> Result<(StatusCode, Json<ProjectWorkspace>), (StatusCode, String)> {
    let project_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid project id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let is_primary = body.is_primary.unwrap_or(false);
    let row = sqlx::query_as::<_, ProjectWorkspace>(
        "INSERT INTO project_workspaces (id, company_id, project_id, name, cwd, repo_url, repo_ref, is_primary, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9) RETURNING id, company_id, project_id, name, cwd, repo_url, repo_ref, metadata, is_primary, created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(project_id)
    .bind(&body.name)
    .bind(&body.cwd)
    .bind(&body.repo_url)
    .bind(&body.repo_ref)
    .bind(is_primary)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// PATCH /api/projects/:id/workspaces/:workspace_id
pub async fn update_workspace(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectWorkspaceIdParam>,
    Json(body): Json<UpdateWorkspaceBody>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, ProjectWorkspace>(
        "UPDATE project_workspaces SET name = COALESCE($2, name), cwd = COALESCE($3, cwd), repo_url = COALESCE($4, repo_url), repo_ref = COALESCE($5, repo_ref), is_primary = COALESCE($6, is_primary), updated_at = $7 WHERE id = $1 AND project_id = $8 RETURNING id, company_id, project_id, name, cwd, repo_url, repo_ref, metadata, is_primary, created_at, updated_at",
    )
    .bind(&params.workspace_id)
    .bind(body.name.as_deref())
    .bind(body.cwd.as_deref())
    .bind(body.repo_url.as_deref())
    .bind(body.repo_ref.as_deref())
    .bind(body.is_primary)
    .bind(now)
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Workspace not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/projects/:id/workspaces/:workspace_id
pub async fn delete_workspace(
    State(pool): State<PgPool>,
    Path(params): Path<ProjectWorkspaceIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM project_workspaces WHERE id = $1 AND project_id = $2")
        .bind(&params.workspace_id)
        .bind(&params.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Workspace not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn workspaces_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set",
    )
}
