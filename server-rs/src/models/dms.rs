//! DMS (Văn bản) response types matching UI api/dms.ts.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmsUser {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmsDocumentPublic {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub r#type: String,
    pub file_size: i32,
    pub file_url: String,
    pub created_at: DateTime<Utc>,
    pub uploaded_by: DmsUser,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmsAssignment {
    pub id: String,
    pub assigned_to: DmsUser,
    pub status: String,
    pub deadline: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmsIncomingDocument {
    pub id: Uuid,
    pub title: String,
    pub document_number: Option<String>,
    pub r#type: String,
    pub status: String,
    pub priority: String,
    pub sender: Option<String>,
    pub received_date: NaiveDate,
    pub deadline: Option<NaiveDate>,
    pub summary: Option<String>,
    pub assignments: Vec<DmsAssignment>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<DmsUser>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmsOutgoingDocument {
    pub id: Uuid,
    pub title: String,
    pub document_number: Option<String>,
    pub status: String,
    pub priority: String,
    pub recipient: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: DmsUser,
}
