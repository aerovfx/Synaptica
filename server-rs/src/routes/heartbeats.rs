//! Heartbeat runs: wakeup, list, get, events (run log), cancel.

use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::heartbeat_run::HeartbeatRun;
use crate::models::heartbeat_run_event::HeartbeatRunEvent;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct RunIdParam {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRunsQuery {
    pub agent_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsQuery {
    pub after_seq: Option<i32>,
    pub limit: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct WakeupBody {
    pub source: Option<String>,
    pub trigger_detail: Option<String>,
    pub reason: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub idempotency_key: Option<String>,
}

/// POST /api/agents/:id/wakeup — enqueue a heartbeat run and start adapter execution (process/http).
pub async fn wakeup_agent(
    State(state): State<crate::routes::ApiState>,
    Path(params): Path<crate::routes::agents::AgentIdParam>,
    Json(body): Json<WakeupBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agent_id =
        Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid agent id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    let status: String = sqlx::query_scalar("SELECT status FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    if status != "idle" && status != "paused" {
        return Ok(Json(serde_json::json!({ "status": "skipped" })));
    }

    let source = body
        .source
        .as_deref()
        .unwrap_or("on_demand")
        .to_string();
    let trigger_detail = body.trigger_detail.clone();

    let run_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    sqlx::query(
        r#"
        INSERT INTO heartbeat_runs (
            id, company_id, agent_id, invocation_source, trigger_detail, status, created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, 'queued', $6, $6)
        "#,
    )
    .bind(run_id)
    .bind(company_id)
    .bind(agent_id)
    .bind(&source)
    .bind(trigger_detail.as_deref())
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row: HeartbeatRun = sqlx::query_as(
        "SELECT id, company_id, agent_id, invocation_source, trigger_detail, status, started_at, finished_at, \
         error, wakeup_request_id, exit_code, signal, usage_json, result_json, session_id_before, session_id_after, \
         log_store, log_ref, log_bytes, log_sha256, log_compressed, stdout_excerpt, stderr_excerpt, error_code, \
         external_run_id, context_snapshot, created_at, updated_at FROM heartbeat_runs WHERE id = $1",
    )
    .bind(run_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::runner::spawn_run(
        state.pool.clone(),
        run_id,
        state.runner_semaphore.clone(),
        state.runner_limits.clone(),
        Some(state.metrics_active_runs.clone()),
    );

    Ok(Json(serde_json::to_value(&row).unwrap()))
}

/// GET /api/companies/:company_id/heartbeat-runs
pub async fn list_runs(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Query(q): Query<ListRunsQuery>,
) -> Result<Json<Vec<HeartbeatRun>>, (StatusCode, String)> {
    let limit = q.limit.unwrap_or(50).min(200);
    let rows = if let Some(ref agent_id) = q.agent_id {
        sqlx::query_as::<_, HeartbeatRun>(
            "SELECT id, company_id, agent_id, invocation_source, trigger_detail, status, started_at, finished_at, \
             error, wakeup_request_id, exit_code, signal, usage_json, result_json, session_id_before, session_id_after, \
             log_store, log_ref, log_bytes, log_sha256, log_compressed, stdout_excerpt, stderr_excerpt, error_code, \
             external_run_id, context_snapshot, created_at, updated_at FROM heartbeat_runs \
             WHERE company_id = $1 AND agent_id = $2 ORDER BY created_at DESC LIMIT $3",
        )
        .bind(&params.company_id)
        .bind(agent_id)
        .bind(limit)
    } else {
        sqlx::query_as::<_, HeartbeatRun>(
            "SELECT id, company_id, agent_id, invocation_source, trigger_detail, status, started_at, finished_at, \
             error, wakeup_request_id, exit_code, signal, usage_json, result_json, session_id_before, session_id_after, \
             log_store, log_ref, log_bytes, log_sha256, log_compressed, stdout_excerpt, stderr_excerpt, error_code, \
             external_run_id, context_snapshot, created_at, updated_at FROM heartbeat_runs \
             WHERE company_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(&params.company_id)
        .bind(limit)
    }
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/heartbeat-runs/:id
pub async fn get_run(
    State(pool): State<PgPool>,
    Path(params): Path<RunIdParam>,
) -> Result<Json<HeartbeatRun>, (StatusCode, String)> {
    let row: Option<HeartbeatRun> = sqlx::query_as(
        "SELECT id, company_id, agent_id, invocation_source, trigger_detail, status, started_at, finished_at, \
         error, wakeup_request_id, exit_code, signal, usage_json, result_json, session_id_before, session_id_after, \
         log_store, log_ref, log_bytes, log_sha256, log_compressed, stdout_excerpt, stderr_excerpt, error_code, \
         external_run_id, context_snapshot, created_at, updated_at FROM heartbeat_runs WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    row.map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Run not found".to_string()))
}

/// GET /api/heartbeat-runs/:id/events — run log (events)
pub async fn list_run_events(
    State(pool): State<PgPool>,
    Path(params): Path<RunIdParam>,
    Query(q): Query<ListEventsQuery>,
) -> Result<Json<Vec<HeartbeatRunEvent>>, (StatusCode, String)> {
    let after_seq = q.after_seq.unwrap_or(0);
    let limit = q.limit.unwrap_or(200).min(500);
    let rows = sqlx::query_as::<_, HeartbeatRunEvent>(
        "SELECT id, company_id, run_id, agent_id, seq, event_type, stream, level, color, message, payload, created_at \
         FROM heartbeat_run_events WHERE run_id = $1 AND seq > $2 ORDER BY seq ASC LIMIT $3",
    )
    .bind(&params.id)
    .bind(after_seq)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/heartbeat-runs/:id/cancel
pub async fn cancel_run(
    State(pool): State<PgPool>,
    Path(params): Path<RunIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let result = sqlx::query(
        "UPDATE heartbeat_runs SET status = 'cancelled', finished_at = $2, updated_at = $2 \
         WHERE id = $1 AND status IN ('queued', 'running')",
    )
    .bind(&params.id)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::CONFLICT, "Run not found or already finished".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunLogResponse {
    pub run_id: String,
    pub store: String,
    pub log_ref: String,
    pub content: String,
    pub next_offset: Option<i64>,
}

/// GET /api/heartbeat-runs/:id/log — stub: returns empty content if no log stored
pub async fn get_run_log(
    State(pool): State<PgPool>,
    Path(params): Path<RunIdParam>,
) -> Result<Json<RunLogResponse>, (StatusCode, String)> {
    let row: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id::text, log_store, log_ref FROM heartbeat_runs WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let (run_id, store, log_ref) = row.ok_or_else(|| (StatusCode::NOT_FOUND, "Run not found".to_string()))?;
    Ok(Json(RunLogResponse {
        run_id,
        store: store.unwrap_or_else(|| "none".to_string()),
        log_ref: log_ref.unwrap_or_default(),
        content: String::new(),
        next_offset: None,
    }))
}
