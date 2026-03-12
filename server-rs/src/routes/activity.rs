use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct IssueIdParam {
    pub id: String,
}

#[derive(Deserialize)]
pub struct RunIdParam {
    pub id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub actor_type: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub agent_id: Option<uuid::Uuid>,
    pub run_id: Option<uuid::Uuid>,
    pub details: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ListActivityQuery {
    pub agent_id: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivityBody {
    pub actor_type: Option<String>,
    pub actor_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub agent_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

/// GET /api/companies/:companyId/activity — optional query: agentId, entityType, entityId
pub async fn list_activity(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Query(query): Query<ListActivityQuery>,
) -> Result<Json<Vec<ActivityEntry>>, (StatusCode, String)> {
    let agent_id: Option<Uuid> = query.agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let rows = sqlx::query_as::<_, ActivityEntry>(
        r#"
        SELECT id, company_id, actor_type, actor_id, action, entity_type, entity_id, agent_id, run_id, details, created_at
        FROM activity_log
        WHERE company_id = $1
          AND (agent_id = $2 OR $2::uuid IS NULL)
          AND (entity_type = $3 OR $3::text IS NULL)
          AND (entity_id = $4 OR $4::text IS NULL)
        ORDER BY created_at DESC
        LIMIT 200
        "#,
    )
    .bind(&params.company_id)
    .bind(agent_id)
    .bind(query.entity_type.as_deref())
    .bind(query.entity_id.as_deref())
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/companies/:companyId/activity
pub async fn create_activity(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateActivityBody>,
) -> Result<(StatusCode, Json<ActivityEntry>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let actor_type = body.actor_type.as_deref().unwrap_or("system");
    let agent_id: Option<Uuid> = body.agent_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, ActivityEntry>(
        "INSERT INTO activity_log (id, company_id, actor_type, actor_id, action, entity_type, entity_id, agent_id, details) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id, company_id, actor_type, actor_id, action, entity_type, entity_id, agent_id, run_id, details, created_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(actor_type)
    .bind(&body.actor_id)
    .bind(&body.action)
    .bind(&body.entity_type)
    .bind(&body.entity_id)
    .bind(agent_id)
    .bind(body.details.as_ref())
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/issues/:id/activity
pub async fn list_issue_activity(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
) -> Result<Json<Vec<ActivityEntry>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ActivityEntry>(
        "SELECT id, company_id, actor_type, actor_id, action, entity_type, entity_id, agent_id, run_id, details, created_at FROM activity_log WHERE entity_type = 'issue' AND entity_id = $1 ORDER BY created_at DESC",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub run_id: Uuid,
    pub status: String,
    pub agent_id: Uuid,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub invocation_source: String,
    pub usage_json: Option<serde_json::Value>,
    pub result_json: Option<serde_json::Value>,
}

/// GET /api/issues/:id/runs
pub async fn list_issue_runs(
    State(pool): State<PgPool>,
    Path(params): Path<IssueIdParam>,
) -> Result<Json<Vec<RunSummary>>, (StatusCode, String)> {
    let issue_id = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid issue id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Issue not found".to_string()))?;
    let issue_id_text = issue_id.to_string();
    let rows = sqlx::query_as::<_, RunSummary>(
        r#"
        SELECT r.id AS run_id, r.status, r.agent_id, r.started_at, r.finished_at, r.created_at, r.invocation_source,
               r.usage_json, r.result_json
        FROM heartbeat_runs r
        WHERE r.company_id = $1
          AND (
            (r.context_snapshot->>'issueId')::uuid = $2
            OR EXISTS (
              SELECT 1 FROM activity_log a
              WHERE a.company_id = r.company_id AND a.entity_type = 'issue' AND a.entity_id = $3 AND a.run_id = r.id
            )
          )
        ORDER BY r.created_at DESC
        "#,
    )
    .bind(company_id)
    .bind(issue_id)
    .bind(&issue_id_text)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RunIssueRef {
    pub issue_id: Uuid,
    pub identifier: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

/// GET /api/heartbeat-runs/:runId/issues
pub async fn list_run_issues(
    State(pool): State<PgPool>,
    Path(params): Path<RunIdParam>,
) -> Result<Json<Vec<RunIssueRef>>, (StatusCode, String)> {
    let run_id = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid run id".to_string()))?;
    let company_id: Option<Uuid> = sqlx::query_scalar("SELECT company_id FROM heartbeat_runs WHERE id = $1")
        .bind(run_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let company_id = match company_id {
        Some(c) => c,
        None => return Ok(Json(Vec::new())),
    };
    let from_activity = sqlx::query_as::<_, RunIssueRef>(
        r#"
        SELECT DISTINCT ON (i.id) i.id AS issue_id, i.identifier, i.title, i.status, i.priority
        FROM activity_log a
        JOIN issues i ON i.company_id = a.company_id AND i.id::text = a.entity_id
        WHERE a.company_id = $1 AND a.run_id = $2 AND a.entity_type = 'issue'
        ORDER BY i.id
        "#,
    )
    .bind(company_id)
    .bind(run_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let context_issue_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT (context_snapshot->>'issueId')::uuid FROM heartbeat_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut out = from_activity;
    if let Some(cid) = context_issue_id {
        if !out.iter().any(|r| r.issue_id == cid) {
            if let Some(row) = sqlx::query_as::<_, RunIssueRef>(
                "SELECT id AS issue_id, identifier, title, status, priority FROM issues WHERE company_id = $1 AND id = $2",
            )
            .bind(company_id)
            .bind(cid)
            .fetch_optional(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            {
                out.insert(0, row);
            }
        }
    }
    Ok(Json(out))
}

pub async fn activity_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set",
    )
}
