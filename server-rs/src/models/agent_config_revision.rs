use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfigRevision {
    pub id: Uuid,
    pub company_id: Uuid,
    pub agent_id: Uuid,
    pub created_by_agent_id: Option<Uuid>,
    pub created_by_user_id: Option<String>,
    pub source: String,
    pub rolled_back_from_revision_id: Option<Uuid>,
    pub changed_keys: serde_json::Value,
    pub before_config: serde_json::Value,
    pub after_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
