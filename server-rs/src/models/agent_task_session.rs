use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentTaskSession {
    pub id: Uuid,
    pub company_id: Uuid,
    pub agent_id: Uuid,
    pub adapter_type: String,
    pub task_key: String,
    pub session_params_json: Option<serde_json::Value>,
    pub session_display_id: Option<String>,
    pub last_run_id: Option<Uuid>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
