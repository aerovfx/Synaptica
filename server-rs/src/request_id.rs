//! Request ID middleware: use client `X-Request-Id` if present, otherwise generate a new UUID.

use axum::http::{header::HeaderName, header::HeaderValue, Request};
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Produces a request ID from the `X-Request-Id` header if valid, otherwise a new UUID v4.
#[derive(Clone, Default)]
pub struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, request: &Request<B>) -> Option<RequestId> {
        let id = request
            .headers()
            .get(&X_REQUEST_ID)
            .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
            .filter(|s: &&str| !s.is_empty())
            .and_then(|s| HeaderValue::from_str(s).ok())
            .map(RequestId::new)
            .unwrap_or_else(|| RequestId::new(HeaderValue::from_str(&Uuid::new_v4().to_string()).unwrap()));
        Some(id)
    }
}

pub fn x_request_id_header_name() -> HeaderName {
    X_REQUEST_ID
}
