use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalComment {
    pub id: Uuid,
    pub company_id: Uuid,
    pub approval_id: Uuid,
    pub author_agent_id: Option<Uuid>,
    pub author_user_id: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
