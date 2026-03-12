use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::goal::Goal;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct GoalIdParam {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalBody {
    pub title: String,
    pub description: Option<String>,
    pub level: Option<String>,
    pub status: Option<String>,
    pub parent_id: Option<String>,
    pub owner_agent_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub level: Option<String>,
    pub status: Option<String>,
    pub parent_id: Option<String>,
    pub owner_agent_id: Option<String>,
}

/// GET /api/companies/:companyId/goals
pub async fn list_goals(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Goal>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Goal>(
        "SELECT id, company_id, title, description, level, status, parent_id, owner_agent_id, created_at, updated_at FROM goals WHERE company_id = $1 ORDER BY created_at",
    )
    .bind(params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/goals/:id
pub async fn get_goal(
    State(pool): State<PgPool>,
    Path(params): Path<GoalIdParam>,
) -> Result<Json<Goal>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Goal>(
        "SELECT id, company_id, title, description, level, status, parent_id, owner_agent_id, created_at, updated_at FROM goals WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Goal not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/goals
pub async fn create_goal(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateGoalBody>,
) -> Result<(StatusCode, Json<Goal>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let level = body.level.as_deref().unwrap_or("task");
    let status = body.status.as_deref().unwrap_or("planned");
    let parent_id: Option<Uuid> = body.parent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let owner_agent_id: Option<Uuid> = body.owner_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, Goal>(
        "INSERT INTO goals (id, company_id, title, description, level, status, parent_id, owner_agent_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9) RETURNING id, company_id, title, description, level, status, parent_id, owner_agent_id, created_at, updated_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(level)
    .bind(status)
    .bind(parent_id)
    .bind(owner_agent_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// PATCH /api/goals/:id
pub async fn update_goal(
    State(pool): State<PgPool>,
    Path(params): Path<GoalIdParam>,
    Json(body): Json<UpdateGoalBody>,
) -> Result<Json<Goal>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Goal>(
        "UPDATE goals SET title = COALESCE($2, title), description = COALESCE($3, description), level = COALESCE($4, level), status = COALESCE($5, status), parent_id = COALESCE($6, parent_id), owner_agent_id = COALESCE($7, owner_agent_id), updated_at = $8 WHERE id = $1 RETURNING id, company_id, title, description, level, status, parent_id, owner_agent_id, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(body.title.as_deref())
    .bind(body.description.as_deref())
    .bind(body.level.as_deref())
    .bind(body.status.as_deref())
    .bind(body.parent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok()))
    .bind(body.owner_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok()))
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Goal not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/goals/:id
pub async fn delete_goal(
    State(pool): State<PgPool>,
    Path(params): Path<GoalIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let r = sqlx::query("DELETE FROM goals WHERE id = $1")
        .bind(&params.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if r.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Goal not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn goals_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
