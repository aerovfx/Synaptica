use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::RequireBoard;
use crate::models::agent::Agent;
use crate::models::agent_api_key::AgentApiKey;
use crate::models::heartbeat_run::HeartbeatRun;
use crate::models::agent_config_revision::AgentConfigRevision;
use crate::models::agent_runtime_state::AgentRuntimeState;
use crate::models::agent_task_session::AgentTaskSession;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct AgentIdParam {
    pub id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentBody {
    pub name: String,
    pub role: Option<String>,
    pub title: Option<String>,
    pub icon: Option<String>,
    pub status: Option<String>,
    pub reports_to: Option<String>,
    pub capabilities: Option<String>,
    pub adapter_type: Option<String>,
    pub adapter_config: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentBody {
    pub name: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
    pub icon: Option<String>,
    pub status: Option<String>,
    pub reports_to: Option<String>,
    pub capabilities: Option<String>,
    pub adapter_type: Option<String>,
    pub adapter_config: Option<serde_json::Value>,
}

/// GET /api/agents/me — identity of current agent (header X-Agent-Id required until auth)
pub async fn get_agent_me(
    State(pool): State<PgPool>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let agent_id = headers
        .get("x-agent-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "X-Agent-Id header required".to_string()))?;
    let row = sqlx::query_as::<_, Agent>(
        "SELECT id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at FROM agents WHERE id = $1",
    )
    .bind(agent_id)
        .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    Ok(Json(row))
}

/// GET /api/companies/:companyId/agents
pub async fn list_agents(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Agent>(
        "SELECT id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at FROM agents WHERE company_id = $1 ORDER BY created_at",
    )
    .bind(params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/agents/:id
pub async fn get_agent(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Agent>(
        "SELECT id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at FROM agents WHERE id = $1",
    )
    .bind(&params.id)
        .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/agents
pub async fn create_agent(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateAgentBody>,
) -> Result<(StatusCode, Json<Agent>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let role = body.role.as_deref().unwrap_or("general");
    let status = body.status.as_deref().unwrap_or("idle");
    let adapter_type = body.adapter_type.as_deref().unwrap_or("process");
    let adapter_config = body.adapter_config.as_ref().cloned().unwrap_or_else(|| json!({}));
    let reports_to: Option<Uuid> = body.reports_to.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, Agent>(
        "INSERT INTO agents (id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14) RETURNING id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(&body.name)
    .bind(role)
    .bind(body.title.as_deref())
    .bind(body.icon.as_deref())
    .bind(status)
    .bind(reports_to)
    .bind(body.capabilities.as_deref())
    .bind(adapter_type)
    .bind(&adapter_config)
    .bind(&json!({}))
    .bind(&json!({}))
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// POST /api/agents/:id/heartbeat — update last_heartbeat_at
pub async fn heartbeat_agent(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Agent>(
        "UPDATE agents SET last_heartbeat_at = $2, updated_at = $2 WHERE id = $1 RETURNING id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(now)
        .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    Ok(Json(row))
}

/// GET /api/agents/:id/config-revisions
pub async fn list_config_revisions(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Vec<AgentConfigRevision>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AgentConfigRevision>(
        "SELECT id, company_id, agent_id, created_by_agent_id, created_by_user_id, source, rolled_back_from_revision_id, changed_keys, before_config, after_config, created_at FROM agent_config_revisions WHERE agent_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/agents/:id/runtime-state
pub async fn get_runtime_state(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<AgentRuntimeState>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, AgentRuntimeState>(
        "SELECT agent_id, company_id, adapter_type, session_id, state_json, last_run_id, last_run_status, total_input_tokens, total_output_tokens, total_cached_input_tokens, total_cost_cents, last_error, created_at, updated_at FROM agent_runtime_state WHERE agent_id = $1",
    )
    .bind(&params.id)
        .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Runtime state not found".to_string()))?;
    Ok(Json(row))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuntimeStateBody {
    pub state_json: Option<serde_json::Value>,
}

/// PATCH /api/agents/:id/runtime-state — upsert runtime state
pub async fn update_runtime_state(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
    Json(body): Json<UpdateRuntimeStateBody>,
) -> Result<Json<AgentRuntimeState>, (StatusCode, String)> {
    let agent_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid agent id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    let (adapter_type,): (String,) = sqlx::query_as("SELECT adapter_type FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let now = chrono::Utc::now();
    let state_json = body.state_json.unwrap_or_else(|| serde_json::json!({}));
    let row = sqlx::query_as::<_, AgentRuntimeState>(
        "INSERT INTO agent_runtime_state (agent_id, company_id, adapter_type, state_json, updated_at) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (agent_id) DO UPDATE SET state_json = COALESCE($4, agent_runtime_state.state_json), updated_at = $5 RETURNING agent_id, company_id, adapter_type, session_id, state_json, last_run_id, last_run_status, total_input_tokens, total_output_tokens, total_cached_input_tokens, total_cost_cents, last_error, created_at, updated_at",
    )
    .bind(agent_id)
    .bind(company_id)
    .bind(&adapter_type)
    .bind(&state_json)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(row))
}

/// GET /api/agents/:id/task-sessions
pub async fn list_task_sessions(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Vec<AgentTaskSession>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AgentTaskSession>(
        "SELECT id, company_id, agent_id, adapter_type, task_key, session_params_json, session_display_id, last_run_id, last_error, created_at, updated_at FROM agent_task_sessions WHERE agent_id = $1 ORDER BY updated_at DESC",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct InvokeBody {
    pub source: Option<String>,
    pub trigger_detail: Option<String>,
    pub reason: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub idempotency_key: Option<String>,
}

/// POST /api/agents/:id/invoke — create a heartbeat run and start adapter execution (process/http).
pub async fn invoke_agent(
    State(state): State<crate::routes::ApiState>,
    Path(params): Path<AgentIdParam>,
    body: Option<Json<InvokeBody>>,
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
        .as_ref()
        .and_then(|b| b.source.as_deref())
        .unwrap_or("on_demand")
        .to_string();
    let trigger_detail = body.as_ref().and_then(|b| b.trigger_detail.clone());

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

/// POST /api/agents/:id/pause
pub async fn pause_agent(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    set_agent_status(&pool, &params.id, "paused").await
}

/// POST /api/agents/:id/resume
pub async fn resume_agent(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    set_agent_status(&pool, &params.id, "idle").await
}

/// POST /api/agents/:id/terminate
pub async fn terminate_agent(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    set_agent_status(&pool, &params.id, "terminated").await
}

async fn set_agent_status(pool: &PgPool, id: &str, status: &str) -> Result<Json<Agent>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Agent>(
        "UPDATE agents SET status = $2, updated_at = $3 WHERE id = $1 RETURNING id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at",
    )
    .bind(id)
    .bind(status)
    .bind(now)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    Ok(Json(row))
}

/// PATCH /api/agents/:id
pub async fn update_agent(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
    Json(body): Json<UpdateAgentBody>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let reports_to: Option<Uuid> = body.reports_to.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let row = sqlx::query_as::<_, Agent>(
        "UPDATE agents SET name = COALESCE($2, name), role = COALESCE($3, role), title = COALESCE($4, title), icon = COALESCE($5, icon), status = COALESCE($6, status), reports_to = COALESCE($7, reports_to), capabilities = COALESCE($8, capabilities), adapter_type = COALESCE($9, adapter_type), adapter_config = COALESCE($10, adapter_config), updated_at = $11 WHERE id = $1 RETURNING id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(body.name.as_deref())
    .bind(body.role.as_deref())
    .bind(body.title.as_deref())
    .bind(body.icon.as_deref())
    .bind(body.status.as_deref())
    .bind(reports_to)
    .bind(body.capabilities.as_deref())
    .bind(body.adapter_type.as_deref())
    .bind(body.adapter_config.as_ref())
    .bind(now)
        .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    Ok(Json(row))
}

#[derive(Deserialize)]
pub struct AgentKeyIdParam {
    pub id: String,
    pub key_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentKeyBody {
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentKeyResponse {
    pub id: String,
    pub name: String,
    pub key: String,
}

/// GET /api/agents/:id/keys — list API keys (no secret)
pub async fn list_agent_keys(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
) -> Result<Json<Vec<AgentApiKey>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AgentApiKey>(
        "SELECT id, agent_id, company_id, name, last_used_at, revoked_at, created_at FROM agent_api_keys WHERE agent_id = $1 AND revoked_at IS NULL ORDER BY created_at",
    )
    .bind(&params.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// POST /api/agents/:id/keys — create key; returns plain key once (store hash only)
pub async fn create_agent_key(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
    Json(body): Json<CreateAgentKeyBody>,
) -> Result<(StatusCode, Json<CreateAgentKeyResponse>), (StatusCode, String)> {
    let agent_id: Uuid = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid agent id".to_string()))?;
    let company_id: Uuid = sqlx::query_scalar("SELECT company_id FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    let key_id = Uuid::new_v4();
    let mut bytes = [0u8; 24];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut bytes);
    let raw = format!(
        "paperclip_{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes)
    );
    let key_hash = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(raw.as_bytes());
        format!("{:x}", h.finalize())
    };
    sqlx::query(
        "INSERT INTO agent_api_keys (id, agent_id, company_id, name, key_hash) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(key_id)
    .bind(agent_id)
    .bind(company_id)
    .bind(&body.name)
    .bind(&key_hash)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(CreateAgentKeyResponse {
            id: key_id.to_string(),
            name: body.name.clone(),
            key: raw,
        }),
    ))
}

/// DELETE /api/agents/:id/keys/:key_id — revoke key
pub async fn revoke_agent_key(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<AgentKeyIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let result = sqlx::query("UPDATE agent_api_keys SET revoked_at = $1 WHERE id = $2 AND agent_id = $3")
        .bind(now)
        .bind(&params.key_id)
        .bind(&params.id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Key not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn agents_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
