use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::board::Sprint;

#[derive(Deserialize)]
pub struct BoardIdParam {
    pub company_id: String,
    pub board_id: String,
}

#[derive(Deserialize)]
pub struct SprintIdParam {
    pub company_id: String,
    pub board_id: String,
    pub sprint_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSprintBody {
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSprintBody {
    pub name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// GET /api/companies/:companyId/boards/:boardId/sprints
pub async fn list_sprints(
    State(pool): State<PgPool>,
    Path(params): Path<BoardIdParam>,
) -> Result<Json<Vec<Sprint>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let rows = sqlx::query_as::<_, Sprint>(
        "SELECT id, board_id, name, start_date, end_date, status, created_at, updated_at FROM sprints WHERE board_id = $1 ORDER BY start_date NULLS LAST, created_at",
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/boards/:boardId/sprints
pub async fn create_sprint(
    State(pool): State<PgPool>,
    Path(params): Path<BoardIdParam>,
    Json(body): Json<CreateSprintBody>,
) -> Result<(StatusCode, Json<Sprint>), (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let id = Uuid::new_v4();
    let status = body.status.as_deref().unwrap_or("planned");
    let start_date: Option<chrono::NaiveDate> = body.start_date.as_deref().and_then(parse_date);
    let end_date: Option<chrono::NaiveDate> = body.end_date.as_deref().and_then(parse_date);
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, Sprint>(
        "INSERT INTO sprints (id, board_id, name, start_date, end_date, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $7) RETURNING id, board_id, name, start_date, end_date, status, created_at, updated_at",
    )
    .bind(id)
    .bind(board_id)
    .bind(&body.name)
    .bind(start_date)
    .bind(end_date)
    .bind(status)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/companies/:companyId/boards/:boardId/sprints/:sprintId
pub async fn get_sprint(
    State(pool): State<PgPool>,
    Path(params): Path<SprintIdParam>,
) -> Result<Json<Sprint>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let sprint_id = Uuid::parse_str(&params.sprint_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid sprint id".to_string()))?;
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let row = sqlx::query_as::<_, Sprint>(
        "SELECT id, board_id, name, start_date, end_date, status, created_at, updated_at FROM sprints WHERE id = $1 AND board_id = $2",
    )
    .bind(sprint_id)
    .bind(board_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Sprint not found".to_string()))?;
    Ok(Json(row))
}

/// PATCH /api/companies/:companyId/boards/:boardId/sprints/:sprintId
pub async fn update_sprint(
    State(pool): State<PgPool>,
    Path(params): Path<SprintIdParam>,
    Json(body): Json<UpdateSprintBody>,
) -> Result<Json<Sprint>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let sprint_id = Uuid::parse_str(&params.sprint_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid sprint id".to_string()))?;
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let start_date: Option<chrono::NaiveDate> = body.start_date.as_deref().and_then(parse_date);
    let end_date: Option<chrono::NaiveDate> = body.end_date.as_deref().and_then(parse_date);
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, Sprint>(
        "UPDATE sprints SET name = COALESCE($2, name), start_date = COALESCE($3, start_date), end_date = COALESCE($4, end_date), status = COALESCE($5, status), updated_at = $6 WHERE id = $1 AND board_id = $7 RETURNING id, board_id, name, start_date, end_date, status, created_at, updated_at",
    )
    .bind(sprint_id)
    .bind(body.name.as_deref())
    .bind(start_date)
    .bind(end_date)
    .bind(body.status.as_deref())
    .bind(now)
    .bind(board_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Sprint not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId/boards/:boardId/sprints/:sprintId
pub async fn delete_sprint(
    State(pool): State<PgPool>,
    Path(params): Path<SprintIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let sprint_id = Uuid::parse_str(&params.sprint_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid sprint id".to_string()))?;
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let result = sqlx::query("DELETE FROM sprints WHERE id = $1 AND board_id = $2")
        .bind(sprint_id)
        .bind(board_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Sprint not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}
