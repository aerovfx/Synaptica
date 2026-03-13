//! DMS (Document Management System) — Văn bản. Company-scoped.

use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::dms::{
    DmsDocumentPublic, DmsIncomingDocument, DmsOutgoingDocument, DmsUser,
};

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmsListAllResponse {
    pub documents: Vec<DmsDocumentPublic>,
    pub incoming: Vec<DmsIncomingDocument>,
    pub outgoing: Vec<DmsOutgoingDocument>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDmsDocumentBody {
    pub title: String,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub file_url: String,
    pub file_size: i32,
    pub uploaded_by_agent_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadDmsDocumentBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub content_base64: String,
    pub content_type: String,
    pub file_name: String,
}

fn dms_user_from_agent(id: Option<Uuid>, name: Option<String>) -> DmsUser {
    let (id_str, first_name) = match (id, name) {
        (Some(u), Some(n)) => (u.to_string(), n),
        (Some(u), None) => (u.to_string(), "—".to_string()),
        (None, _) => ("".to_string(), "—".to_string()),
    };
    DmsUser {
        id: id_str,
        first_name: first_name,
        last_name: String::new(),
        avatar: None,
    }
}

/// GET /api/companies/:companyId/dms/documents — tài liệu chung
pub async fn list_dms_documents(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<DmsDocumentPublic>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;

    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        #[allow(dead_code)]
        company_id: Uuid,
        title: String,
        description: Option<String>,
        #[sqlx(rename = "type")]
        type_: String,
        file_size: i32,
        file_url: String,
        uploaded_by_agent_id: Option<Uuid>,
        created_at: chrono::DateTime<Utc>,
        uploaded_by_name: Option<String>,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT d.id, d.company_id, d.title, d.description, d.type, d.file_size, d.file_url,
               d.uploaded_by_agent_id, d.created_at, a.name AS uploaded_by_name
        FROM company_dms_documents d
        LEFT JOIN agents a ON d.uploaded_by_agent_id = a.id
        WHERE d.company_id = $1
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let list: Vec<DmsDocumentPublic> = rows
        .into_iter()
        .map(|r| DmsDocumentPublic {
            id: r.id,
            title: r.title,
            description: r.description,
            r#type: r.type_,
            file_size: r.file_size,
            file_url: r.file_url.clone(),
            created_at: r.created_at,
            uploaded_by: dms_user_from_agent(r.uploaded_by_agent_id, r.uploaded_by_name),
        })
        .collect();
    Ok(Json(list))
}

/// POST /api/companies/:companyId/dms/documents — tạo tài liệu (metadata + fileUrl có sẵn)
pub async fn create_dms_document(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateDmsDocumentBody>,
) -> Result<(StatusCode, Json<DmsDocumentPublic>), (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let uploaded_by_agent_id = body
        .uploaded_by_agent_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let doc_type = body.r#type.as_deref().unwrap_or("OTHER");
    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO company_dms_documents (id, company_id, title, description, type, file_size, file_url, uploaded_by_agent_id, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(id)
    .bind(company_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(doc_type)
    .bind(body.file_size)
    .bind(&body.file_url)
    .bind(uploaded_by_agent_id)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let uploaded_by_name: Option<String> = match uploaded_by_agent_id {
        Some(aid) => sqlx::query_scalar::<_, Option<String>>("SELECT name FROM agents WHERE id = $1")
            .bind(aid)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten()
            .flatten(),
        None => None,
    };
    let doc = DmsDocumentPublic {
        id,
        title: body.title,
        description: body.description,
        r#type: doc_type.to_string(),
        file_size: body.file_size,
        file_url: body.file_url,
        created_at: now,
        uploaded_by: dms_user_from_agent(uploaded_by_agent_id, uploaded_by_name),
    };
    Ok((StatusCode::CREATED, Json(doc)))
}

/// POST /api/companies/:companyId/dms/documents/upload — upload file (base64) + tạo tài liệu và asset
pub async fn upload_dms_document(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<UploadDmsDocumentBody>,
) -> Result<(StatusCode, Json<DmsDocumentPublic>), (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;

    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &body.content_base64,
    )
    .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid base64 content".to_string()))?;
    let file_size = bytes.len() as i32;

    let asset_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let provider = "local";
    let object_key = format!("{}/{}", params.company_id, asset_id);
    let sha256_hex = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&bytes);
        format!("{:x}", h.finalize())
    };

    if let Ok(base) = std::env::var("ASSETS_PATH") {
        let path = std::path::Path::new(&base).join(&object_key);
        if let Some(p) = path.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let _ = std::fs::write(&path, &bytes);
    }

    sqlx::query(
        r#"INSERT INTO assets (id, company_id, provider, object_key, content_type, byte_size, sha256, original_filename, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)"#,
    )
    .bind(asset_id)
    .bind(&params.company_id)
    .bind(provider)
    .bind(&object_key)
    .bind(&body.content_type)
    .bind(file_size)
    .bind(&sha256_hex)
    .bind(&body.file_name)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let file_url = format!("/api/assets/{}/content", asset_id);
    let title = body
        .title
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| body.file_name.clone());
    let doc_type = body.r#type.as_deref().unwrap_or("OTHER");
    let doc_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO company_dms_documents (id, company_id, title, description, type, file_size, file_url, uploaded_by_agent_id, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, $8)"#,
    )
    .bind(doc_id)
    .bind(company_id)
    .bind(&title)
    .bind(body.description.as_deref())
    .bind(doc_type)
    .bind(file_size)
    .bind(&file_url)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let doc = DmsDocumentPublic {
        id: doc_id,
        title,
        description: body.description,
        r#type: doc_type.to_string(),
        file_size,
        file_url,
        created_at: now,
        uploaded_by: dms_user_from_agent(None, None),
    };
    Ok((StatusCode::CREATED, Json(doc)))
}

/// GET /api/companies/:companyId/dms/incoming — văn bản đến
pub async fn list_dms_incoming(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<DmsIncomingDocument>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;

    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        #[allow(dead_code)]
        company_id: Uuid,
        title: String,
        document_number: Option<String>,
        #[sqlx(rename = "type")]
        type_: String,
        status: String,
        priority: String,
        sender: Option<String>,
        received_date: NaiveDate,
        deadline: Option<NaiveDate>,
        summary: Option<String>,
        created_by_agent_id: Option<Uuid>,
        created_at: chrono::DateTime<Utc>,
        created_by_name: Option<String>,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT d.id, d.company_id, d.title, d.document_number, d.type, d.status, d.priority,
               d.sender, d.received_date, d.deadline, d.summary, d.created_by_agent_id, d.created_at,
               a.name AS created_by_name
        FROM company_dms_incoming d
        LEFT JOIN agents a ON d.created_by_agent_id = a.id
        WHERE d.company_id = $1
        ORDER BY d.received_date DESC, d.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let list: Vec<DmsIncomingDocument> = rows
        .into_iter()
        .map(|r| DmsIncomingDocument {
            id: r.id,
            title: r.title,
            document_number: r.document_number,
            r#type: r.type_,
            status: r.status,
            priority: r.priority,
            sender: r.sender,
            received_date: r.received_date,
            deadline: r.deadline,
            summary: r.summary,
            assignments: vec![], // Phase 5 minimal: no assignments table yet
            created_at: r.created_at,
            created_by: match (r.created_by_agent_id, r.created_by_name) {
                (None, _) => None,
                (id, name) => Some(dms_user_from_agent(id, name)),
            },
        })
        .collect();
    Ok(Json(list))
}

/// GET /api/companies/:companyId/dms/outgoing — văn bản đi
pub async fn list_dms_outgoing(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<DmsOutgoingDocument>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;

    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        #[allow(dead_code)]
        company_id: Uuid,
        title: String,
        document_number: Option<String>,
        status: String,
        priority: String,
        recipient: Option<String>,
        created_by_agent_id: Option<Uuid>,
        created_at: chrono::DateTime<Utc>,
        created_by_name: Option<String>,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT d.id, d.company_id, d.title, d.document_number, d.status, d.priority, d.recipient,
               d.created_by_agent_id, d.created_at, a.name AS created_by_name
        FROM company_dms_outgoing d
        LEFT JOIN agents a ON d.created_by_agent_id = a.id
        WHERE d.company_id = $1
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let list: Vec<DmsOutgoingDocument> = rows
        .into_iter()
        .map(|r| DmsOutgoingDocument {
            id: r.id,
            title: r.title,
            document_number: r.document_number,
            status: r.status,
            priority: r.priority,
            recipient: r.recipient,
            created_at: r.created_at,
            created_by: dms_user_from_agent(r.created_by_agent_id, r.created_by_name),
        })
        .collect();
    Ok(Json(list))
}

/// GET /api/companies/:companyId/dms — combined list (documents + incoming + outgoing) in one round-trip.
pub async fn list_dms_all(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<DmsListAllResponse>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;

    let (doc_res, inc_res, out_res) = tokio::join!(
        fetch_documents(&pool, company_id),
        fetch_incoming(&pool, company_id),
        fetch_outgoing(&pool, company_id),
    );

    let documents = doc_res.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let incoming = inc_res.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let outgoing = out_res.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(DmsListAllResponse {
        documents,
        incoming,
        outgoing,
    }))
}

async fn fetch_documents(
    pool: &PgPool,
    company_id: Uuid,
) -> Result<Vec<DmsDocumentPublic>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        title: String,
        description: Option<String>,
        #[sqlx(rename = "type")]
        type_: String,
        file_size: i32,
        file_url: String,
        uploaded_by_agent_id: Option<Uuid>,
        created_at: chrono::DateTime<Utc>,
        uploaded_by_name: Option<String>,
    }
    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT d.id, d.company_id, d.title, d.description, d.type, d.file_size, d.file_url,
               d.uploaded_by_agent_id, d.created_at, a.name AS uploaded_by_name
        FROM company_dms_documents d
        LEFT JOIN agents a ON d.uploaded_by_agent_id = a.id
        WHERE d.company_id = $1
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| DmsDocumentPublic {
            id: r.id,
            title: r.title,
            description: r.description,
            r#type: r.type_,
            file_size: r.file_size,
            file_url: r.file_url,
            created_at: r.created_at,
            uploaded_by: dms_user_from_agent(r.uploaded_by_agent_id, r.uploaded_by_name),
        })
        .collect())
}

async fn fetch_incoming(
    pool: &PgPool,
    company_id: Uuid,
) -> Result<Vec<DmsIncomingDocument>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        title: String,
        document_number: Option<String>,
        #[sqlx(rename = "type")]
        type_: String,
        status: String,
        priority: String,
        sender: Option<String>,
        received_date: NaiveDate,
        deadline: Option<NaiveDate>,
        summary: Option<String>,
        created_by_agent_id: Option<Uuid>,
        created_at: chrono::DateTime<Utc>,
        created_by_name: Option<String>,
    }
    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT d.id, d.company_id, d.title, d.document_number, d.type, d.status, d.priority,
               d.sender, d.received_date, d.deadline, d.summary, d.created_by_agent_id, d.created_at,
               a.name AS created_by_name
        FROM company_dms_incoming d
        LEFT JOIN agents a ON d.created_by_agent_id = a.id
        WHERE d.company_id = $1
        ORDER BY d.received_date DESC, d.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| DmsIncomingDocument {
            id: r.id,
            title: r.title,
            document_number: r.document_number,
            r#type: r.type_,
            status: r.status,
            priority: r.priority,
            sender: r.sender,
            received_date: r.received_date,
            deadline: r.deadline,
            summary: r.summary,
            assignments: vec![],
            created_at: r.created_at,
            created_by: match (r.created_by_agent_id, r.created_by_name) {
                (None, _) => None,
                (id, name) => Some(dms_user_from_agent(id, name)),
            },
        })
        .collect())
}

async fn fetch_outgoing(
    pool: &PgPool,
    company_id: Uuid,
) -> Result<Vec<DmsOutgoingDocument>, String> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        title: String,
        document_number: Option<String>,
        status: String,
        priority: String,
        recipient: Option<String>,
        created_by_agent_id: Option<Uuid>,
        created_at: chrono::DateTime<Utc>,
        created_by_name: Option<String>,
    }
    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT d.id, d.company_id, d.title, d.document_number, d.status, d.priority, d.recipient,
               d.created_by_agent_id, d.created_at, a.name AS created_by_name
        FROM company_dms_outgoing d
        LEFT JOIN agents a ON d.created_by_agent_id = a.id
        WHERE d.company_id = $1
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|r| DmsOutgoingDocument {
            id: r.id,
            title: r.title,
            document_number: r.document_number,
            status: r.status,
            priority: r.priority,
            recipient: r.recipient,
            created_at: r.created_at,
            created_by: dms_user_from_agent(r.created_by_agent_id, r.created_by_name),
        })
        .collect())
}

pub async fn dms_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set; use Node server or set DATABASE_URL",
    )
}
