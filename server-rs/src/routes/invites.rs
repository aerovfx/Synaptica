use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
pub struct TokenParam {
    pub token: String,
}

#[derive(Deserialize)]
pub struct InviteIdParam {
    pub invite_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteBody {
    pub invite_type: Option<String>,
    pub allowed_join_types: Option<String>,
    pub expires_in_days: Option<i64>,
}

fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    format!("{:x}", h.finalize())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InviteRow {
    pub id: Uuid,
    pub company_id: Option<Uuid>,
    pub invite_type: String,
    pub allowed_join_types: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub accepted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInviteResponse {
    pub id: Uuid,
    pub token: String,
    pub invite_type: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// GET /api/companies/:company_id/invites
pub async fn list_invites(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<InviteRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, InviteRow>(
        "SELECT id, company_id, invite_type, allowed_join_types, expires_at, revoked_at, accepted_at, created_at FROM invites WHERE company_id = $1 ORDER BY created_at DESC",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

/// GET /api/invites/:token — get invite by token (token hashed for lookup)
pub async fn get_invite_by_token(
    State(pool): State<PgPool>,
    Path(params): Path<TokenParam>,
) -> Result<Json<InviteRow>, (StatusCode, String)> {
    let token_hash = hash_token(&params.token);
    let row = sqlx::query_as::<_, InviteRow>(
        "SELECT id, company_id, invite_type, allowed_join_types, expires_at, revoked_at, accepted_at, created_at FROM invites WHERE token_hash = $1 AND (revoked_at IS NULL) AND (expires_at > now())",
    )
    .bind(&token_hash)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Invite not found or expired".to_string()))?;
    Ok(Json(row))
}

/// POST /api/invites/:inviteId/revoke
pub async fn revoke_invite(
    State(pool): State<PgPool>,
    Path(params): Path<InviteIdParam>,
) -> Result<Json<InviteRow>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, InviteRow>(
        "UPDATE invites SET revoked_at = $2, updated_at = $2 WHERE id = $1 AND revoked_at IS NULL RETURNING id, company_id, invite_type, allowed_join_types, expires_at, revoked_at, accepted_at, created_at",
    )
    .bind(&params.invite_id)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Invite not found or already revoked".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:company_id/invites
pub async fn create_invite(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateInviteBody>,
) -> Result<(StatusCode, Json<CreateInviteResponse>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let days = body.expires_in_days.unwrap_or(7);
    let expires_at = now + chrono::Duration::days(days);
    let invite_type = body.invite_type.as_deref().unwrap_or("company_join");
    let allowed_join_types = body.allowed_join_types.as_deref().unwrap_or("both");
    let token = format!("inv_{}", Uuid::new_v4().simple());
    let token_hash = hash_token(&token);
    sqlx::query(
        "INSERT INTO invites (id, company_id, invite_type, token_hash, allowed_join_types, expires_at, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(invite_type)
    .bind(&token_hash)
    .bind(allowed_join_types)
    .bind(expires_at)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(CreateInviteResponse {
            id,
            token,
            invite_type: invite_type.to_string(),
            expires_at,
        }),
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingManifest {
    invite: InviteRow,
    onboarding: OnboardingDetails,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingDetails {
    invite_message: Option<String>,
    connectivity: Option<ConnectivityInfo>,
    text_instructions: Option<TextInstructions>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectivityInfo {
    guidance: Option<String>,
    connection_candidates: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextInstructions {
    url: String,
}

/// GET /api/invites/:token/onboarding — JSON manifest for invite onboarding
pub async fn get_invite_onboarding(
    State(pool): State<PgPool>,
    Path(params): Path<TokenParam>,
) -> Result<Json<OnboardingManifest>, (StatusCode, String)> {
    let token_hash = hash_token(&params.token);
    let invite = sqlx::query_as::<_, InviteRow>(
        "SELECT id, company_id, invite_type, allowed_join_types, expires_at, revoked_at, accepted_at, created_at FROM invites WHERE token_hash = $1 AND (revoked_at IS NULL) AND (expires_at > now())",
    )
    .bind(&token_hash)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Invite not found or expired".to_string()))?;
    let manifest = OnboardingManifest {
        onboarding: OnboardingDetails {
            invite_message: None,
            connectivity: Some(ConnectivityInfo {
                guidance: Some("Verify Paperclip is reachable from your runtime with: GET <base-url>/api/health".to_string()),
                connection_candidates: vec![],
            }),
            text_instructions: Some(TextInstructions {
                url: format!("/api/invites/{}/onboarding.txt", params.token),
            }),
        },
        invite,
    };
    Ok(Json(manifest))
}

/// GET /api/invites/:token/onboarding.txt — plain-text onboarding instructions for agents
pub async fn get_invite_onboarding_txt(
    State(pool): State<PgPool>,
    Path(params): Path<TokenParam>,
) -> Result<(StatusCode, [(axum::http::HeaderName, &'static str); 1], String), (StatusCode, String)> {
    let token_hash = hash_token(&params.token);
    let invite = sqlx::query_as::<_, InviteRow>(
        "SELECT id, company_id, invite_type, allowed_join_types, expires_at, revoked_at, accepted_at, created_at FROM invites WHERE token_hash = $1 AND (revoked_at IS NULL) AND (expires_at > now())",
    )
    .bind(&token_hash)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Invite not found or expired".to_string()))?;
    let company_id = invite.company_id.map(|id| id.to_string()).unwrap_or_default();
    let token = &params.token;
    let text = format!(
        "# Paperclip Agent Onboarding\n\nYou have been invited to join a Paperclip organization as an agent.\n\nInvite type:  {invite_type}\nAllowed join: agents\nExpires:      {expires}\nCompany ID:   {company_id}\n\n## How to join\n\n1. Submit a join request to:\n   POST /api/invites/{token}/accept\n\n   with JSON body:\n   {{\n     \"requestType\": \"agent\",\n     \"agentName\": \"<your agent name>\",\n     \"adapterType\": \"<your adapter type>\",\n     \"agentDefaultsPayload\": {{\n       \"url\": \"<your callback or gateway URL>\"\n     }}\n   }}\n\n   Supported adapterType values: openclaw_gateway, openfang_gateway, http, process\n\n2. If board approval is required, your request will be pending.\n   You will receive a claim secret when approved.\n   If no approval is required, you receive it immediately.\n\n3. Claim your API key:\n   POST /api/join-requests/<requestId>/claim-api-key\n   {{ \"claimSecret\": \"<secret from step 2>\" }}\n\n4. Use the returned token as Bearer in Authorization header for all subsequent calls.\n\n5. Review agent docs at: /api/llms/agent-configuration.txt\n\n## Connectivity check\n\nBefore submitting, verify the server is reachable:\n  GET /api/health\n  Expected: {{ \"status\": \"ok\" }}\n",
        invite_type = invite.invite_type,
        expires = invite.expires_at.format("%Y-%m-%dT%H:%M:%SZ"),
        company_id = company_id,
        token = token,
    );
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        text,
    ))
}

pub async fn invites_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
