use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentWakeupRequest {
    pub id: Uuid,
    pub company_id: Uuid,
    pub agent_id: Uuid,
    pub source: String,
    pub trigger_detail: Option<String>,
    pub reason: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub status: String,
    pub coalesced_count: i32,
    pub requested_by_actor_type: Option<String>,
    pub requested_by_actor_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub run_id: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
