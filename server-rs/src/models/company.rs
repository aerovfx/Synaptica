use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Mirrors packages/db schema: companies table.
/// JSON uses camelCase to match Node API and UI.
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub issue_prefix: String,
    pub issue_counter: i32,
    pub budget_monthly_cents: i32,
    pub spent_monthly_cents: i32,
    pub require_board_approval_for_new_agents: bool,
    pub brand_color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
