use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRunEvent {
    pub id: i64,
    pub company_id: Uuid,
    pub run_id: Uuid,
    pub agent_id: Uuid,
    pub seq: i32,
    pub event_type: String,
    pub stream: Option<String>,
    pub level: Option<String>,
    pub color: Option<String>,
    pub message: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
