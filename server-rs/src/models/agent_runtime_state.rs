use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentRuntimeState {
    pub agent_id: Uuid,
    pub company_id: Uuid,
    pub adapter_type: String,
    pub session_id: Option<String>,
    pub state_json: serde_json::Value,
    pub last_run_id: Option<Uuid>,
    pub last_run_status: Option<String>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cached_input_tokens: i64,
    pub total_cost_cents: i64,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
