use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::approval::Approval;
use crate::models::issue::Issue;
use crate::models::issue_attachment::IssueAttachment;
use crate::models::issue_comment::IssueComment;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct IssueIdParam {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueBody {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project_id: Option<String>,
    pub goal_id: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project_id: Option<String>,
    pub goal_id: Option<String>,
    pub assignee_agent_id: Option<String>,
}

/// GET /api/companies/:companyId/issues
pub async fn list_issues(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Issue>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Issue>(
        "SELECT id, company_id, project_id, goal_id, parent_id, title, description, status, priority, assignee_agent_id, assignee_user_id, checkout_run_id, execution_run_id, execution_agent_name_key, execution_locked_at, created_by_agent_id, created_by_user_id, issue_number, identifier, request_depth, billing_code, assignee_adapter_overrides, execution_workspace_settings, started_at, completed_at, cancelled_at, hidden_at, created_at, updated_at FROM issues WHERE company_id = $1 ORDER BY created_at",
    )
    .bind(params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/issues/:id
pub async fn get_issue(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
) -> Result<Json<Issue>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Issue>(
        "SELECT id, company_id, project_id, goal_id, parent_id, title, description, status, priority, assignee_agent_id, assignee_user_id, checkout_run_id, execution_run_id, execution_agent_name_key, execution_locked_at, created_by_agent_id, created_by_user_id, issue_number, identifier, request_depth, billing_code, assignee_adapter_overrides, execution_workspace_settings, started_at, completed_at, cancelled_at, hidden_at, created_at, updated_at FROM issues WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    Ok(Json(row))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IssueReadState {
    pub id: Uuid,
    pub company_id: Uuid,
    pub issue_id: Uuid,
    pub user_id: String,
    pub last_read_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkReadBody {
    pub user_id: Option<String>,
}

/// POST /api/issues/:id/read — mark issue as read for a user (board context).
pub async fn mark_issue_read(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
    Json(body): Json<Option<MarkReadBody>>,
) -> Result<Json<IssueReadState>, (StatusCode, String)> {
    let issue_id = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid issue id".to_string()))?;
    let (company_id,): (Uuid,) = sqlx::query_as(
        "SELECT company_id FROM issues WHERE id = $1",
    )
    .bind(issue_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    let user_id = body
        .and_then(|b| b.user_id)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "board".to_string());
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, IssueReadState>(
        r#"
        INSERT INTO issue_read_states (id, company_id, issue_id, user_id, last_read_at, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $4, $4)
        ON CONFLICT (company_id, issue_id, user_id) DO UPDATE SET last_read_at = $4, updated_at = $4
        RETURNING id, company_id, issue_id, user_id, last_read_at, created_at, updated_at
        "#,
    )
    .bind(company_id)
    .bind(issue_id)
    .bind(&user_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/issues
pub async fn create_issue(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateIssueBody>,
) -> Result<(StatusCode, Json<Issue>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let status = body.status.as_deref().unwrap_or("backlog");
    let priority = body.priority.as_deref().unwrap_or("medium");
    let project_id: Option<Uuid> = body.project_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let goal_id: Option<Uuid> = body.goal_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let parent_id: Option<Uuid> = body.parent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, Issue>(
        "INSERT INTO issues (id, company_id, project_id, goal_id, parent_id, title, description, status, priority, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10) RETURNING id, company_id, project_id, goal_id, parent_id, title, description, status, priority, assignee_agent_id, assignee_user_id, checkout_run_id, execution_run_id, execution_agent_name_key, execution_locked_at, created_by_agent_id, created_by_user_id, issue_number, identifier, request_depth, billing_code, assignee_adapter_overrides, execution_workspace_settings, started_at, completed_at, cancelled_at, hidden_at, created_at, updated_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(project_id)
    .bind(goal_id)
    .bind(parent_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(status)
    .bind(priority)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutIssueBody {
    pub assignee_agent_id: Option<String>,
}

/// POST /api/issues/:id/checkout — claim + start (idempotent if same assignee)
pub async fn checkout_issue(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Option<CheckoutIssueBody>>,
) -> Result<Json<Issue>, (StatusCode, String)> {
    let assignee_agent_id: Uuid = body
        .as_ref()
        .and_then(|b| b.assignee_agent_id.as_ref())
        .and_then(|s| Uuid::parse_str(s).ok())
        .or_else(|| {
            headers
                .get("x-agent-id")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok())
        })
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "assignee_agent_id or X-Agent-Id required".to_string()))?;
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Issue>(
        "UPDATE issues SET assignee_agent_id = $2, status = 'in_progress', started_at = COALESCE(started_at, $3), updated_at = $3 WHERE id = $1 AND (assignee_agent_id IS NULL OR assignee_agent_id = $2) RETURNING id, company_id, project_id, goal_id, parent_id, title, description, status, priority, assignee_agent_id, assignee_user_id, checkout_run_id, execution_run_id, execution_agent_name_key, execution_locked_at, created_by_agent_id, created_by_user_id, issue_number, identifier, request_depth, billing_code, assignee_adapter_overrides, execution_workspace_settings, started_at, completed_at, cancelled_at, hidden_at, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(assignee_agent_id)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::CONFLICT, "Issue not found or already assigned to another agent".to_string()))?;
    Ok(Json(row))
}

/// POST /api/issues/:id/release
pub async fn release_issue(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
) -> Result<Json<Issue>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Issue>(
        "UPDATE issues SET assignee_agent_id = NULL, assignee_user_id = NULL, status = 'backlog', started_at = NULL, execution_locked_at = NULL, updated_at = $2 WHERE id = $1 RETURNING id, company_id, project_id, goal_id, parent_id, title, description, status, priority, assignee_agent_id, assignee_user_id, checkout_run_id, execution_run_id, execution_agent_name_key, execution_locked_at, created_by_agent_id, created_by_user_id, issue_number, identifier, request_depth, billing_code, assignee_adapter_overrides, execution_workspace_settings, started_at, completed_at, cancelled_at, hidden_at, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    Ok(Json(row))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueCommentBody {
    pub body: String,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
}

/// GET /api/issues/:id/comments
pub async fn list_issue_comments(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
) -> Result<Json<Vec<IssueComment>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, IssueComment>(
        "SELECT id, company_id, issue_id, author_agent_id, author_user_id, body, created_at, updated_at FROM issue_comments WHERE issue_id = $1 ORDER BY created_at",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/issues/:id/comments
pub async fn add_issue_comment(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
    Json(body): Json<CreateIssueCommentBody>,
) -> Result<(StatusCode, Json<IssueComment>), (StatusCode, String)> {
    let issue_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid issue id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    let author_agent_id: Option<Uuid> = body.author_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, IssueComment>(
        "INSERT INTO issue_comments (id, company_id, issue_id, author_agent_id, author_user_id, body, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $7) RETURNING id, company_id, issue_id, author_agent_id, author_user_id, body, created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(issue_id)
    .bind(author_agent_id)
    .bind(body.author_user_id.as_deref())
    .bind(&body.body)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/issues/:id/approvals — approvals linked to this issue
pub async fn list_issue_approvals(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
) -> Result<Json<Vec<Approval>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Approval>(
        "SELECT a.id, a.company_id, a.\"type\", a.requested_by_agent_id, a.requested_by_user_id, a.status, a.payload, a.decision_note, a.decided_by_user_id, a.decided_at, a.created_at, a.updated_at FROM approvals a INNER JOIN issue_approvals ia ON ia.approval_id = a.id WHERE ia.issue_id = $1 ORDER BY a.created_at DESC",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkIssueApprovalBody {
    pub approval_id: String,
}

/// POST /api/issues/:id/approvals — link approval to issue
pub async fn link_issue_approval(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
    Json(body): Json<LinkIssueApprovalBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    let issue_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid issue id".to_string()))?;
    let approval_id: Uuid = Uuid::parse_str(&body.approval_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid approval_id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    let _: Uuid = sqlx::query_scalar("SELECT id FROM approvals WHERE id = $1 AND company_id = $2")
        .bind(approval_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Approval not found".to_string()))?;
    sqlx::query(
        "INSERT INTO issue_approvals (company_id, issue_id, approval_id) VALUES ($1, $2, $3) ON CONFLICT (issue_id, approval_id) DO NOTHING",
    )
    .bind(company_id)
    .bind(issue_id)
    .bind(approval_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct IssueApprovalIdParam {
    pub id: String,
    pub approval_id: String,
}

/// GET /api/issues/:id/attachments
pub async fn list_issue_attachments(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
) -> Result<Json<Vec<IssueAttachment>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, IssueAttachment>(
        "SELECT id, company_id, issue_id, asset_id, issue_comment_id, created_at, updated_at FROM issue_attachments WHERE issue_id = $1 ORDER BY created_at",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkIssueAttachmentBody {
    pub asset_id: String,
    pub issue_comment_id: Option<String>,
}

/// POST /api/issues/:id/attachments — link asset to issue
pub async fn add_issue_attachment(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
    Json(body): Json<LinkIssueAttachmentBody>,
) -> Result<(StatusCode, Json<IssueAttachment>), (StatusCode, String)> {
    let issue_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid issue id".to_string()))?;
    let asset_id: Uuid = Uuid::parse_str(&body.asset_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid asset_id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    let _: Uuid = sqlx::query_scalar("SELECT id FROM assets WHERE id = $1 AND company_id = $2")
        .bind(asset_id)
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Asset not found".to_string()))?;
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let issue_comment_id: Option<Uuid> = body.issue_comment_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, IssueAttachment>(
        "INSERT INTO issue_attachments (id, company_id, issue_id, asset_id, issue_comment_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6) RETURNING id, company_id, issue_id, asset_id, issue_comment_id, created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(issue_id)
    .bind(asset_id)
    .bind(issue_comment_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Deserialize)]
pub struct IssueAttachmentIdParam {
    pub id: String,
    pub attachment_id: String,
}

/// DELETE /api/issues/:id/attachments/:attachment_id
pub async fn delete_issue_attachment(
    State(pool): State<PgPool>,
    Path(params): Path<IssueAttachmentIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM issue_attachments WHERE id = $1 AND issue_id = $2")
        .bind(&params.attachment_id)
        .bind(&params.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Attachment not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/issues/:id/approvals/:approval_id
pub async fn unlink_issue_approval(
    State(pool): State<PgPool>,
    Path(params): Path<IssueApprovalIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM issue_approvals WHERE issue_id = $1 AND approval_id = $2")
        .bind(&params.id)
        .bind(&params.approval_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Link not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/issues/:id
pub async fn update_issue(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
    Json(body): Json<UpdateIssueBody>,
) -> Result<Json<Issue>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let project_id: Option<Uuid> = body.project_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let goal_id: Option<Uuid> = body.goal_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let assignee_agent_id: Option<Uuid> = body.assignee_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, Issue>(
        "UPDATE issues SET title = COALESCE($2, title), description = COALESCE($3, description), status = COALESCE($4, status), priority = COALESCE($5, priority), project_id = COALESCE($6, project_id), goal_id = COALESCE($7, goal_id), assignee_agent_id = COALESCE($8, assignee_agent_id), updated_at = $9 WHERE id = $1 RETURNING id, company_id, project_id, goal_id, parent_id, title, description, status, priority, assignee_agent_id, assignee_user_id, checkout_run_id, execution_run_id, execution_agent_name_key, execution_locked_at, created_by_agent_id, created_by_user_id, issue_number, identifier, request_depth, billing_code, assignee_adapter_overrides, execution_workspace_settings, started_at, completed_at, cancelled_at, hidden_at, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(body.title.as_deref())
    .bind(body.description.as_deref())
    .bind(body.status.as_deref())
    .bind(body.priority.as_deref())
    .bind(project_id)
    .bind(goal_id)
    .bind(assignee_agent_id)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    Ok(Json(row))
}

pub async fn issues_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
