use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::board::{Board, BoardColumn};

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct BoardIdParam {
    pub company_id: String,
    pub board_id: String,
}

#[derive(Deserialize)]
pub struct ColumnIdParam {
    pub company_id: String,
    pub board_id: String,
    pub column_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardBody {
    pub name: String,
    pub project_id: Option<String>,
    #[serde(rename = "type")]
    pub board_type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBoardBody {
    pub name: Option<String>,
    pub project_id: Option<String>,
    #[serde(rename = "type")]
    pub board_type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardColumnBody {
    pub name: String,
    pub position: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBoardColumnBody {
    pub name: Option<String>,
    pub position: Option<f32>,
}

/// GET /api/companies/:companyId/boards
pub async fn list_boards(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Board>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, Board>(
        "SELECT id, company_id, project_id, name, type as board_type, created_at, updated_at FROM boards WHERE company_id = $1 ORDER BY created_at",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/boards
pub async fn create_board(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateBoardBody>,
) -> Result<(StatusCode, Json<Board>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let project_id: Option<Uuid> = body.project_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let board_type = body.board_type.as_deref().unwrap_or("kanban");
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, Board>(
        "INSERT INTO boards (id, company_id, project_id, name, type, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6) RETURNING id, company_id, project_id, name, type as board_type, created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(project_id)
    .bind(&body.name)
    .bind(board_type)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/companies/:companyId/boards/:boardId
pub async fn get_board(
    State(pool): State<PgPool>,
    Path(params): Path<BoardIdParam>,
) -> Result<Json<Board>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let row = sqlx::query_as::<_, Board>(
        "SELECT id, company_id, project_id, name, type as board_type, created_at, updated_at FROM boards WHERE id = $1 AND company_id = $2",
    )
    .bind(board_id)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;
    Ok(Json(row))
}

/// PATCH /api/companies/:companyId/boards/:boardId
pub async fn update_board(
    State(pool): State<PgPool>,
    Path(params): Path<BoardIdParam>,
    Json(body): Json<UpdateBoardBody>,
) -> Result<Json<Board>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let project_id: Option<Uuid> = body.project_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, Board>(
        "UPDATE boards SET name = COALESCE($2, name), project_id = COALESCE($3, project_id), type = COALESCE($4, type), updated_at = $5 WHERE id = $1 AND company_id = $6 RETURNING id, company_id, project_id, name, type as board_type, created_at, updated_at",
    )
    .bind(board_id)
    .bind(body.name.as_deref())
    .bind(project_id)
    .bind(body.board_type.as_deref())
    .bind(now)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId/boards/:boardId
pub async fn delete_board(
    State(pool): State<PgPool>,
    Path(params): Path<BoardIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let result = sqlx::query("DELETE FROM boards WHERE id = $1 AND company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Board not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/companies/:companyId/boards/:boardId/columns
pub async fn list_board_columns(
    State(pool): State<PgPool>,
    Path(params): Path<BoardIdParam>,
) -> Result<Json<Vec<BoardColumn>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    // Ensure board belongs to company
    let _: (Uuid,) = sqlx::query_as("SELECT id FROM boards WHERE id = $1 AND company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let rows = sqlx::query_as::<_, BoardColumn>(
        "SELECT id, board_id, name, position, created_at, updated_at FROM board_columns WHERE board_id = $1 ORDER BY position, created_at",
    )
    .bind(board_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/boards/:boardId/columns
pub async fn create_board_column(
    State(pool): State<PgPool>,
    Path(params): Path<BoardIdParam>,
    Json(body): Json<CreateBoardColumnBody>,
) -> Result<(StatusCode, Json<BoardColumn>), (StatusCode, String)> {
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
    let position = body.position.unwrap_or(0.0);
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, BoardColumn>(
        "INSERT INTO board_columns (id, board_id, name, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5) RETURNING id, board_id, name, position, created_at, updated_at",
    )
    .bind(id)
    .bind(board_id)
    .bind(&body.name)
    .bind(position)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// PATCH /api/companies/:companyId/boards/:boardId/columns/:columnId
pub async fn update_board_column(
    State(pool): State<PgPool>,
    Path(params): Path<ColumnIdParam>,
    Json(body): Json<UpdateBoardColumnBody>,
) -> Result<Json<BoardColumn>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let column_id = Uuid::parse_str(&params.column_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid column id".to_string()))?;
    let _: (Uuid,) = sqlx::query_as("SELECT b.id FROM boards b WHERE b.id = $1 AND b.company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, BoardColumn>(
        "UPDATE board_columns SET name = COALESCE($2, name), position = COALESCE($3, position), updated_at = $4 WHERE id = $1 AND board_id = $5 RETURNING id, board_id, name, position, created_at, updated_at",
    )
    .bind(column_id)
    .bind(body.name.as_deref())
    .bind(body.position)
    .bind(now)
    .bind(board_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Column not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId/boards/:boardId/columns/:columnId
pub async fn delete_board_column(
    State(pool): State<PgPool>,
    Path(params): Path<ColumnIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let board_id = Uuid::parse_str(&params.board_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid board id".to_string()))?;
    let column_id = Uuid::parse_str(&params.column_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid column id".to_string()))?;
    let _: (Uuid,) = sqlx::query_as("SELECT b.id FROM boards b WHERE b.id = $1 AND b.company_id = $2")
        .bind(board_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found".to_string()))?;

    let result = sqlx::query("DELETE FROM board_columns WHERE id = $1 AND board_id = $2")
        .bind(column_id)
        .bind(board_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Column not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}
