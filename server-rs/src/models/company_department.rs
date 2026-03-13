use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyDepartment {
    pub id: Uuid,
    pub company_id: Uuid,
    pub space_id: Option<Uuid>,
    pub name: String,
    pub leader_agent_id: Option<Uuid>,
    pub order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
