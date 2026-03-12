use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspace {
    pub id: Uuid,
    pub company_id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub cwd: Option<String>,
    pub repo_url: Option<String>,
    pub repo_ref: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
