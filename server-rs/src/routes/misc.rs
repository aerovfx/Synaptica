use axum::extract::Path;
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidebarBadgesResponse {
    pub pending_approvals: i64,
    pub open_issues: i64,
}

/// GET /api/companies/:company_id/sidebar-badges
pub async fn sidebar_badges(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<SidebarBadgesResponse>, (StatusCode, String)> {
    let company_id = Uuid::parse_str(&params.company_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company id".to_string()))?;
    let pending_approvals: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM approvals WHERE company_id = $1 AND status = 'pending'",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("sidebar_badges (approvals) failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    let open_issues: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM issues WHERE company_id = $1 AND status NOT IN ('done', 'cancelled')",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("sidebar_badges (issues) failed: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(SidebarBadgesResponse {
        pending_approvals,
        open_issues,
    }))
}

/// GET /api/llm-config — LLM config text (from env PAPERCLIP_LLM_CONFIG or empty)
pub async fn llm_config() -> String {
    std::env::var("PAPERCLIP_LLM_CONFIG").unwrap_or_default()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillIndexEntry {
    pub id: String,
    pub name: String,
}

/// GET /api/skills/index — list available skill documents
pub async fn skills_index() -> Json<Vec<SkillIndexEntry>> {
    Json(vec![
        SkillIndexEntry { id: "paperclip".to_string(), name: "Paperclip".to_string() },
        SkillIndexEntry { id: "paperclip-create-agent".to_string(), name: "Paperclip Create Agent".to_string() },
    ])
}

#[derive(Deserialize)]
pub struct SkillIdParam {
    pub id: String,
}

/// GET /api/skills/:id — skill document (markdown); requires SKILLS_DIR env or returns 404
pub async fn get_skill(
    Path(params): Path<SkillIdParam>,
) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    let base = std::env::var("SKILLS_DIR").unwrap_or_else(|_| "skills".to_string());
    let path = std::path::Path::new(&base).join(&params.id).join("SKILL.md");
    let content = std::fs::read_to_string(&path).map_err(|_| (StatusCode::NOT_FOUND, "Skill not found".to_string()))?;
    Ok((
        [(CONTENT_TYPE, "text/plain; charset=utf-8")],
        content,
    ))
}

/// POST /api/board/claim — stub: in authenticated mode would establish board session
pub async fn board_claim() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true, "message": "Board context; no session in local_trusted mode" }))
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct BoardClaimTokenParam {
    pub token: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardClaimChallengeResponse {
    pub status: String,
    pub requires_sign_in: bool,
    pub expires_at: Option<String>,
    pub claimed_by_user_id: Option<String>,
}

/// GET /api/board-claim/:token — inspect board claim challenge (stub: returns invalid when no challenge).
pub async fn get_board_claim(
    Path(params): Path<BoardClaimTokenParam>,
) -> Json<BoardClaimChallengeResponse> {
    let _ = params;
    Json(BoardClaimChallengeResponse {
        status: "invalid".to_string(),
        requires_sign_in: true,
        expires_at: None,
        claimed_by_user_id: None,
    })
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct BoardClaimClaimBody {
    pub code: Option<String>,
}

/// POST /api/board-claim/:token/claim — claim board ownership (stub: 404 no challenge in Rust).
pub async fn post_board_claim_claim(
    Path(params): Path<BoardClaimTokenParam>,
    Json(_body): Json<Option<BoardClaimClaimBody>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let _ = params;
    Err((axum::http::StatusCode::NOT_FOUND, "Board claim challenge not found".to_string()))
}

/// GET /api/auth/get-session — board session; local_trusted returns null (no login required)
pub async fn get_session() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "data": null }))
}

pub async fn sidebar_badges_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set")
}
