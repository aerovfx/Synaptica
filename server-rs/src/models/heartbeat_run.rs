use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRun {
    pub id: Uuid,
    pub company_id: Uuid,
    pub agent_id: Uuid,
    pub invocation_source: String,
    pub trigger_detail: Option<String>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub wakeup_request_id: Option<Uuid>,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
    pub usage_json: Option<serde_json::Value>,
    pub result_json: Option<serde_json::Value>,
    pub session_id_before: Option<String>,
    pub session_id_after: Option<String>,
    pub log_store: Option<String>,
    pub log_ref: Option<String>,
    pub log_bytes: Option<i64>,
    pub log_sha256: Option<String>,
    pub log_compressed: bool,
    pub stdout_excerpt: Option<String>,
    pub stderr_excerpt: Option<String>,
    pub error_code: Option<String>,
    pub external_run_id: Option<String>,
    pub context_snapshot: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
