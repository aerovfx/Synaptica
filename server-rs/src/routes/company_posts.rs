use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::company_post::CompanyPost;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct CompanyPostIdParam {
    pub company_id: String,
    pub post_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyPostBody {
    pub content: String,
    pub author_agent_id: Option<String>,
    pub scheduled_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyPostBody {
    pub content: Option<String>,
    pub scheduled_at: Option<String>,
}

/// GET /api/companies/:companyId/posts (feed)
pub async fn list_company_posts(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CompanyPost>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, CompanyPost>(
        "SELECT id, company_id, author_agent_id, content, scheduled_at, created_at, updated_at FROM company_posts WHERE company_id = $1 ORDER BY created_at DESC",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/posts
pub async fn create_company_post(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateCompanyPostBody>,
) -> Result<(StatusCode, Json<CompanyPost>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let author_agent_id: Option<Uuid> = body
        .author_agent_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let scheduled_at: Option<chrono::DateTime<chrono::Utc>> = body
        .scheduled_at
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanyPost>(
        "INSERT INTO company_posts (id, company_id, author_agent_id, content, scheduled_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6) RETURNING id, company_id, author_agent_id, content, scheduled_at, created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(author_agent_id)
    .bind(body.content.trim())
    .bind(scheduled_at)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/companies/:companyId/posts/:postId
pub async fn get_company_post(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyPostIdParam>,
) -> Result<Json<CompanyPost>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let post_id = Uuid::parse_str(&params.post_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid post id".to_string()))?;
    let row = sqlx::query_as::<_, CompanyPost>(
        "SELECT id, company_id, author_agent_id, content, scheduled_at, created_at, updated_at FROM company_posts WHERE id = $1 AND company_id = $2",
    )
    .bind(post_id)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;
    Ok(Json(row))
}

/// PATCH /api/companies/:companyId/posts/:postId
pub async fn update_company_post(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyPostIdParam>,
    Json(body): Json<UpdateCompanyPostBody>,
) -> Result<Json<CompanyPost>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let post_id = Uuid::parse_str(&params.post_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid post id".to_string()))?;
    let scheduled_at: Option<chrono::DateTime<chrono::Utc>> = body
        .scheduled_at
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let now = chrono::Utc::now();

    let row = sqlx::query_as::<_, CompanyPost>(
        "UPDATE company_posts SET content = COALESCE($2, content), scheduled_at = COALESCE($3, scheduled_at), updated_at = $4 WHERE id = $1 AND company_id = $5 RETURNING id, company_id, author_agent_id, content, scheduled_at, created_at, updated_at",
    )
    .bind(post_id)
    .bind(body.content.as_deref())
    .bind(scheduled_at)
    .bind(now)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Post not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId/posts/:postId
pub async fn delete_company_post(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyPostIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let post_id = Uuid::parse_str(&params.post_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid post id".to_string()))?;

    let result = sqlx::query("DELETE FROM company_posts WHERE id = $1 AND company_id = $2")
        .bind(post_id)
        .bind(company_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Post not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn company_posts_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
