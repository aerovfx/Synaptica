use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: Uuid,
    pub company_id: Uuid,
    pub project_id: Option<Uuid>,
    pub goal_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_agent_id: Option<Uuid>,
    pub assignee_user_id: Option<String>,
    pub checkout_run_id: Option<Uuid>,
    pub execution_run_id: Option<Uuid>,
    pub execution_agent_name_key: Option<String>,
    pub execution_locked_at: Option<DateTime<Utc>>,
    pub created_by_agent_id: Option<Uuid>,
    pub created_by_user_id: Option<String>,
    pub issue_number: Option<i32>,
    pub identifier: Option<String>,
    pub request_depth: i32,
    pub billing_code: Option<String>,
    pub assignee_adapter_overrides: Option<serde_json::Value>,
    pub execution_workspace_settings: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub hidden_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
