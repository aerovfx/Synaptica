use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanySecret {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub provider: String,
    pub external_ref: Option<String>,
    pub latest_version: i32,
    pub description: Option<String>,
    pub created_by_agent_id: Option<Uuid>,
    pub created_by_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
