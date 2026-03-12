//! Auth: actor resolution (board vs agent API key) and board-only guard.

use axum::extract::FromRequestParts;
use axum::extract::Request;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// Caller identity: board (session or local_trusted) or agent (Bearer API key).
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Actor {
    Board,
    Agent {
        agent_id: Uuid,
        company_id: Uuid,
    },
}

impl Actor {
    pub fn is_agent(&self) -> bool {
        matches!(self, Actor::Agent { .. })
    }

    #[allow(dead_code)]
    pub fn agent_company_id(&self) -> Option<Uuid> {
        match self {
            Actor::Agent { company_id, .. } => Some(*company_id),
            _ => None,
        }
    }
}

/// Hash raw API key the same way as when creating (SHA-256 hex).
fn hash_api_key(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    format!("{:x}", h.finalize())
}

/// Resolve actor from request: no/invalid Bearer → Board; valid Bearer → Agent or 401.
pub async fn resolve_actor(
    pool: &PgPool,
    auth_header: Option<&axum::http::HeaderValue>,
) -> Result<Actor, StatusCode> {
    let Some(header_value) = auth_header else {
        return Ok(Actor::Board);
    };
    let Ok(s) = header_value.to_str() else {
        return Ok(Actor::Board);
    };
    let raw = s.trim_start_matches("Bearer ").trim();
    if raw.is_empty() {
        return Ok(Actor::Board);
    }
    let key_hash = hash_api_key(raw);
    let row: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT agent_id, company_id FROM agent_api_keys WHERE key_hash = $1 AND revoked_at IS NULL",
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match row {
        Some((agent_id, company_id)) => {
            // Optionally update last_used_at (fire-and-forget to avoid slowing the request)
            let pool = pool.clone();
            let key_hash = key_hash.clone();
            tokio::spawn(async move {
                let _ = sqlx::query("UPDATE agent_api_keys SET last_used_at = now() WHERE key_hash = $1")
                    .bind(&key_hash)
                    .execute(&pool)
                    .await;
            });
            Ok(Actor::Agent { agent_id, company_id })
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware: resolve actor from Authorization header and insert into extensions.
pub async fn actor_middleware(
    axum::extract::State(pool): axum::extract::State<PgPool>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth = request.headers().get(axum::http::header::AUTHORIZATION);
    match resolve_actor(&pool, auth).await {
        Ok(actor) => {
            request.extensions_mut().insert(actor);
            next.run(request).await
        }
        Err(status) => (status, "Invalid or expired API key").into_response(),
    }
}

/// Extractor that returns 403 if the request is authenticated as an agent (board-only routes).
#[derive(Clone, Copy, Debug)]
pub struct RequireBoard;

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireBoard
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let actor = parts
            .extensions
            .get::<Actor>()
            .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Missing actor").into_response())?;
        if actor.is_agent() {
            return Err((
                StatusCode::FORBIDDEN,
                "This action is only allowed for the board, not for agents",
            )
                .into_response());
        }
        Ok(RequireBoard)
    }
}
