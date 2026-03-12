use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IssueAttachment {
    pub id: Uuid,
    pub company_id: Uuid,
    pub issue_id: Uuid,
    pub asset_id: Uuid,
    pub issue_comment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
