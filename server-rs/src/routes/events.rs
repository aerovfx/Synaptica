//! Realtime: WebSocket GET /api/companies/:company_id/events/ws for live events.

use axum::extract::ws::{Message, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

use crate::auth;
use crate::routes::ApiState;
use uuid::Uuid;

/// In-memory bus: one broadcast channel per company. Subscribers receive LiveEvent JSON.
pub struct LiveEventBus {
    channels: RwLock<HashMap<String, broadcast::Sender<serde_json::Value>>>,
}

impl LiveEventBus {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    /// Subscribe to events for a company. Returns a receiver; drop to unsubscribe.
    pub async fn subscribe(&self, company_id: &str) -> broadcast::Receiver<serde_json::Value> {
        let mut guard = self.channels.write().await;
        let entry = guard.entry(company_id.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            tx
        });
        entry.subscribe()
    }

    /// Publish an event to all subscribers of the company. No-op if no channel exists.
    #[allow(dead_code)]
    pub fn publish(&self, company_id: &str, event: serde_json::Value) {
        let guard = self.channels.try_read();
        if let Ok(channels) = guard {
            if let Some(tx) = channels.get(company_id) {
                let _ = tx.send(event);
            }
        }
    }
}

impl Default for LiveEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize)]
pub struct CompanyIdPath {
    pub company_id: String,
}

/// GET /api/companies/:company_id/events/ws — WebSocket upgrade for company live events.
pub async fn company_events_ws(
    State(state): State<ApiState>,
    Path(params): Path<CompanyIdPath>,
    ws: WebSocketUpgrade,
) -> Result<Response, (StatusCode, String)> {
    let company_id = params.company_id.clone();
    let pool = &state.pool;
    // For WebSocket we don't get headers from WebSocketUpgrade; treat as board (local_trusted). Agent auth could use ?token= or a prior cookie.
    let auth_header = None::<&axum::http::HeaderValue>;
    let actor = auth::resolve_actor(pool, auth_header)
        .await
        .map_err(|e| (e, "Invalid or expired API key".to_string()))?;
    if let auth::Actor::Agent { company_id: agent_cid, .. } = &actor {
        let cid = Uuid::parse_str(&company_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid company_id".to_string()))?;
        if *agent_cid != cid {
            return Err((StatusCode::FORBIDDEN, "Agent may only subscribe to its company".to_string()));
        }
    }
    let live_bus = state.live_bus.clone();
    let company_id_ws = company_id.clone();
    let response = ws.on_upgrade(move |socket| async move {
        let mut rx = live_bus.subscribe(&company_id_ws).await;
        let (mut send_sink, mut recv_stream) = socket.split();
        loop {
            tokio::select! {
                Ok(val) = rx.recv() => {
                    if let Ok(text) = serde_json::to_string(&val) {
                        if send_sink.send(Message::Text(text.into())).await.is_err() { break; }
                    }
                }
                Some(Ok(msg)) = recv_stream.next() => {
                    if let Message::Ping(data) = msg {
                        if send_sink.send(Message::Pong(data)).await.is_err() { break; }
                    }
                }
                else => break,
            }
        }
    });
    Ok(response)
}

/// No-DB stub: GET /api/companies/:company_id/events/ws returns 503.
pub async fn company_events_ws_no_db() -> (axum::http::StatusCode, &'static str) {
    (axum::http::StatusCode::SERVICE_UNAVAILABLE, "Database not configured")
}