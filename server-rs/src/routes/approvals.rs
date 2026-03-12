use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::RequireBoard;
use crate::models::approval::Approval;
use crate::models::approval_comment::ApprovalComment;
use crate::models::issue::Issue;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct ApprovalIdParam {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveRejectBody {
    pub decision_note: Option<String>,
    pub decided_by_user_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalBody {
    pub r#type: String,
    pub payload: serde_json::Value,
    pub requested_by_agent_id: Option<String>,
    pub requested_by_user_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalCommentBody {
    pub body: String,
    pub author_agent_id: Option<String>,
    pub author_user_id: Option<String>,
}

/// GET /api/companies/:companyId/approvals
pub async fn list_approvals(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Approval>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Approval>(
        "SELECT id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at FROM approvals WHERE company_id = $1 ORDER BY created_at DESC",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/approvals/:id
pub async fn get_approval(
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
) -> Result<Json<Approval>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Approval>(
        "SELECT id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at FROM approvals WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Approval not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/approvals/:id/approve
pub async fn approve_approval(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
    Json(body): Json<ApproveRejectBody>,
) -> Result<Json<Approval>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Approval>(
        "UPDATE approvals SET status = 'approved', decision_note = COALESCE($2, decision_note), decided_by_user_id = COALESCE($3, decided_by_user_id), decided_at = $4, updated_at = $4 WHERE id = $1 AND status = 'pending' RETURNING id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(body.decision_note.as_deref())
    .bind(body.decided_by_user_id.as_deref())
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::CONFLICT, "Approval not found or not pending".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/approvals
pub async fn create_approval(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateApprovalBody>,
) -> Result<(StatusCode, Json<Approval>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let requested_by_agent_id: Option<Uuid> = body.requested_by_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, Approval>(
        "INSERT INTO approvals (id, company_id, \"type\", payload, requested_by_agent_id, requested_by_user_id, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $7) RETURNING id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(&body.r#type)
    .bind(&body.payload)
    .bind(requested_by_agent_id)
    .bind(body.requested_by_user_id.as_deref())
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// POST /api/approvals/:id/request-revision
pub async fn request_revision_approval(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
) -> Result<Json<Approval>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Approval>(
        "UPDATE approvals SET status = 'revision_requested', updated_at = $2 WHERE id = $1 AND status = 'pending' RETURNING id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::CONFLICT, "Approval not found or not pending".to_string()))?;
    Ok(Json(row))
}

/// POST /api/approvals/:id/resubmit
pub async fn resubmit_approval(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
    Json(body): Json<Option<serde_json::Value>>,
) -> Result<Json<Approval>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let res = if let Some(payload) = body {
        sqlx::query_as::<_, Approval>(
            "UPDATE approvals SET status = 'pending', payload = $2, decision_note = NULL, decided_by_user_id = NULL, decided_at = NULL, updated_at = $3 WHERE id = $1 AND status IN ('revision_requested', 'rejected') RETURNING id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at",
        )
        .bind(&params.id)
        .bind(&payload)
        .bind(now)
        .fetch_optional(&pool)
        .await
    } else {
        sqlx::query_as::<_, Approval>(
            "UPDATE approvals SET status = 'pending', decision_note = NULL, decided_by_user_id = NULL, decided_at = NULL, updated_at = $2 WHERE id = $1 AND status IN ('revision_requested', 'rejected') RETURNING id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at",
        )
        .bind(&params.id)
        .bind(now)
        .fetch_optional(&pool)
        .await
    };
    let row = res
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::CONFLICT, "Approval not found or not in revision_requested/rejected".to_string()))?;
    Ok(Json(row))
}

/// GET /api/approvals/:id/comments
pub async fn list_approval_comments(
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
) -> Result<Json<Vec<ApprovalComment>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ApprovalComment>(
        "SELECT id, company_id, approval_id, author_agent_id, author_user_id, body, created_at, updated_at FROM approval_comments WHERE approval_id = $1 ORDER BY created_at",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/approvals/:id/comments
pub async fn add_approval_comment(
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
    Json(body): Json<CreateApprovalCommentBody>,
) -> Result<(StatusCode, Json<ApprovalComment>), (StatusCode, String)> {
    let approval_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid approval id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM approvals WHERE id = $1")
        .bind(approval_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Approval not found".to_string()))?;
    let author_agent_id: Option<Uuid> = body.author_agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, ApprovalComment>(
        "INSERT INTO approval_comments (id, company_id, approval_id, author_agent_id, author_user_id, body, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $7) RETURNING id, company_id, approval_id, author_agent_id, author_user_id, body, created_at, updated_at",
    )
    .bind(id)
    .bind(company_id)
    .bind(approval_id)
    .bind(author_agent_id)
    .bind(body.author_user_id.as_deref())
    .bind(&body.body)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/approvals/:id/issues — issues linked to this approval
pub async fn list_approval_issues(
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
) -> Result<Json<Vec<Issue>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Issue>(
        "SELECT i.id, i.company_id, i.project_id, i.goal_id, i.parent_id, i.title, i.description, i.status, i.priority, i.assignee_agent_id, i.assignee_user_id, i.checkout_run_id, i.execution_run_id, i.execution_agent_name_key, i.execution_locked_at, i.created_by_agent_id, i.created_by_user_id, i.issue_number, i.identifier, i.request_depth, i.billing_code, i.assignee_adapter_overrides, i.execution_workspace_settings, i.started_at, i.completed_at, i.cancelled_at, i.hidden_at, i.created_at, i.updated_at FROM issues i INNER JOIN issue_approvals ia ON ia.issue_id = i.id WHERE ia.approval_id = $1 ORDER BY i.created_at",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/approvals/:id/reject
pub async fn reject_approval(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<ApprovalIdParam>,
    Json(body): Json<ApproveRejectBody>,
) -> Result<Json<Approval>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Approval>(
        "UPDATE approvals SET status = 'rejected', decision_note = COALESCE($2, decision_note), decided_by_user_id = COALESCE($3, decided_by_user_id), decided_at = $4, updated_at = $4 WHERE id = $1 AND status = 'pending' RETURNING id, company_id, \"type\", requested_by_agent_id, requested_by_user_id, status, payload, decision_note, decided_by_user_id, decided_at, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(body.decision_note.as_deref())
    .bind(body.decided_by_user_id.as_deref())
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::CONFLICT, "Approval not found or not pending".to_string()))?;
    Ok(Json(row))
}

pub async fn approvals_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set",
    )
}
