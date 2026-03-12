use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Uuid,
    pub company_id: Uuid,
    pub goal_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub lead_agent_id: Option<Uuid>,
    pub target_date: Option<chrono::NaiveDate>,
    pub color: Option<String>,
    pub execution_workspace_policy: Option<serde_json::Value>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
