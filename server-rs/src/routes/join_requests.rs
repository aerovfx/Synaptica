use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct JoinRequestRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub invite_id: Uuid,
    pub request_type: String,
    pub status: String,
    pub agent_name: Option<String>,
    pub adapter_type: Option<String>,
    pub created_agent_id: Option<Uuid>,
    pub approved_by_user_id: Option<String>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejected_by_user_id: Option<String>,
    pub rejected_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct JoinRequestIdParam {
    pub id: String,
}

#[derive(Deserialize)]
pub struct CompanyAndRequestIdParam {
    pub company_id: String,
    pub request_id: String,
}

/// GET /api/companies/:company_id/join-requests
pub async fn list_join_requests(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<JoinRequestRow>>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let rows = sqlx::query_as::<_, JoinRequestRow>(
        "SELECT id, company_id, invite_id, request_type, status, agent_name, adapter_type, created_agent_id, approved_by_user_id, approved_at, rejected_by_user_id, rejected_at, created_at, updated_at FROM join_requests WHERE company_id = $1 ORDER BY created_at DESC",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("GET /api/companies/:company_id/join-requests failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(rows))
}

/// GET /api/join-requests/:id
pub async fn get_join_request(
    State(pool): State<PgPool>,
    Path(params): Path<JoinRequestIdParam>,
) -> Result<Json<JoinRequestRow>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, JoinRequestRow>(
        "SELECT id, company_id, invite_id, request_type, status, agent_name, adapter_type, created_agent_id, approved_by_user_id, approved_at, rejected_by_user_id, rejected_at, created_at, updated_at FROM join_requests WHERE id = $1",
    )
    .bind(&params.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Join request not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/join-requests/:requestId/approve
pub async fn approve_join_request(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyAndRequestIdParam>,
) -> Result<Json<JoinRequestRow>, (StatusCode, String)> {
    let request_id = Uuid::parse_str(&params.request_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid request id".to_string()))?;
    let company_id = Uuid::parse_str(&params.company_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let jr: (Uuid, Uuid, String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT id, company_id, request_type, requesting_user_id, agent_name FROM join_requests WHERE id = $1 AND company_id = $2 AND status = 'pending_approval'",
    )
    .bind(request_id)
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Join request not found or not pending".to_string()))?;
    let (_id, _cid, request_type, requesting_user_id, agent_name) = jr;
    let now = chrono::Utc::now();
    let created_agent_id: Option<Uuid> = if request_type == "agent" {
        let ceo_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM agents WHERE company_id = $1 AND role = 'ceo' AND status != 'terminated' LIMIT 1",
        )
        .bind(company_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let agent_id = Uuid::new_v4();
        let name = agent_name.unwrap_or_else(|| "New Agent".to_string());
        sqlx::query(
            "INSERT INTO agents (id, company_id, name, role, status, reports_to, adapter_type, adapter_config, runtime_config) VALUES ($1, $2, $3, 'general', 'idle', $4, 'process', '{}', '{}')",
        )
        .bind(agent_id)
        .bind(company_id)
        .bind(&name)
        .bind(ceo_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        sqlx::query(
            "INSERT INTO company_memberships (id, company_id, principal_type, principal_id, status, membership_role, updated_at) VALUES (gen_random_uuid(), $1, 'agent', $2, 'active', 'member', $3) ON CONFLICT (company_id, principal_type, principal_id) DO UPDATE SET status = 'active', updated_at = $3",
        )
        .bind(company_id)
        .bind(agent_id)
        .bind(now)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Some(agent_id)
    } else {
        if let Some(ref uid) = requesting_user_id {
            let _ = sqlx::query(
                "INSERT INTO company_memberships (id, company_id, principal_type, principal_id, status, membership_role, updated_at) VALUES (gen_random_uuid(), $1, 'user', $2, 'active', 'member', $3) ON CONFLICT (company_id, principal_type, principal_id) DO UPDATE SET status = 'active', updated_at = $3",
            )
            .bind(company_id)
            .bind(uid)
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
        None
    };
    let row = sqlx::query_as::<_, JoinRequestRow>(
        "UPDATE join_requests SET status = 'approved', approved_by_user_id = $3, approved_at = $4, created_agent_id = $5, updated_at = $4 WHERE id = $1 AND company_id = $2 RETURNING id, company_id, invite_id, request_type, status, agent_name, adapter_type, created_agent_id, approved_by_user_id, approved_at, rejected_by_user_id, rejected_at, created_at, updated_at",
    )
    .bind(request_id)
    .bind(company_id)
    .bind("board")
    .bind(now)
    .bind(created_agent_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Join request not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/join-requests/:requestId/reject
pub async fn reject_join_request(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyAndRequestIdParam>,
) -> Result<Json<JoinRequestRow>, (StatusCode, String)> {
    let request_id = Uuid::parse_str(&params.request_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid request id".to_string()))?;
    let company_id = Uuid::parse_str(&params.company_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, JoinRequestRow>(
        "UPDATE join_requests SET status = 'rejected', rejected_by_user_id = $3, rejected_at = $4, updated_at = $4 WHERE id = $1 AND company_id = $2 AND status = 'pending_approval' RETURNING id, company_id, invite_id, request_type, status, agent_name, adapter_type, created_agent_id, approved_by_user_id, approved_at, rejected_by_user_id, rejected_at, created_at, updated_at",
    )
    .bind(request_id)
    .bind(company_id)
    .bind("board")
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Join request not found or not pending".to_string()))?;
    Ok(Json(row))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimApiKeyBody {
    pub claim_secret: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimApiKeyResponse {
    pub key_id: Uuid,
    pub token: String,
    pub agent_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/join-requests/:requestId/claim-api-key
pub async fn claim_join_request_api_key(
    State(pool): State<PgPool>,
    Path(params): Path<JoinRequestIdParam>,
    Json(body): Json<ClaimApiKeyBody>,
) -> Result<(StatusCode, Json<ClaimApiKeyResponse>), (StatusCode, String)> {
    let request_id = Uuid::parse_str(&params.id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid request id".to_string()))?;
    let claim_hash = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(body.claim_secret.as_bytes());
        format!("{:x}", h.finalize())
    };
    let jr: (Uuid, Option<Uuid>, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT company_id, created_agent_id, claim_secret_consumed_at FROM join_requests WHERE id = $1 AND request_type = 'agent' AND status = 'approved' AND claim_secret_hash = $2",
    )
    .bind(request_id)
    .bind(&claim_hash)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Join request not found, not approved, or claim secret invalid".to_string()))?;
    let (company_id, created_agent_id, consumed) = jr;
    let agent_id = created_agent_id.ok_or_else(|| (StatusCode::CONFLICT, "Join request has no created agent".to_string()))?;
    if consumed.is_some() {
        return Err((StatusCode::CONFLICT, "Claim secret already used".to_string()));
    }
    let key_id = Uuid::new_v4();
    let mut bytes = [0u8; 24];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut bytes);
    let token = format!(
        "paperclip_{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes),
    );
    let key_hash = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(token.as_bytes());
        format!("{:x}", h.finalize())
    };
    let now = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO agent_api_keys (id, agent_id, company_id, name, key_hash) VALUES ($1, $2, $3, 'initial-join-key', $4)",
    )
    .bind(key_id)
    .bind(agent_id)
    .bind(company_id)
    .bind(&key_hash)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query(
        "UPDATE join_requests SET claim_secret_consumed_at = $2, updated_at = $2 WHERE id = $1",
    )
    .bind(request_id)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(ClaimApiKeyResponse {
            key_id,
            token,
            agent_id,
            created_at: now,
        }),
    ))
}

pub async fn join_requests_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
