use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CostEvent {
    pub id: Uuid,
    pub company_id: Uuid,
    pub agent_id: Uuid,
    pub issue_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub goal_id: Option<Uuid>,
    pub billing_code: Option<String>,
    pub provider: String,
    pub model: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub cost_cents: i32,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
