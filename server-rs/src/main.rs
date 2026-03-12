mod auth;
mod config;
mod db;
mod metrics;
mod models;
mod request_id;
mod routes;
mod runner;
mod scheduler;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use axum::http::header::{HeaderName, HeaderValue};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::request_id::{SetRequestIdLayer, PropagateRequestIdLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use request_id::{UuidRequestId, x_request_id_header_name};
use routes::api_routes;

fn static_fallback_service(ui_dist: PathBuf) -> ServeDir<ServeFile> {
    let index_path = ui_dist.join("index.html");
    ServeDir::new(&ui_dist).fallback(ServeFile::new(index_path))
}

fn static_router(ui_dist: PathBuf) -> Router {
    Router::new().nest_service("/", static_fallback_service(ui_dist))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let addr = SocketAddr::from((config.host.as_str().parse::<std::net::IpAddr>()?, config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let x_req_id = x_request_id_header_name();
    let cors_layer = if config.cors_origins.is_empty() {
        CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)
    } else {
        let origins: Vec<HeaderValue> = config
            .cors_origins
            .iter()
            .filter_map(|s| HeaderValue::from_str(s).ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    if let Some(ref database_url) = config.database_url {
        let pool = db::create_pool(database_url, &config).await?;
        tracing::info!("PostgreSQL connected (pool max_size={})", config.db_pool_max_size);
        let pool_scheduler = pool.clone();
        let scheduler_interval_secs = config.scheduler_interval_secs;
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(scheduler_interval_secs));
            loop {
                interval.tick().await;
                if let Err(e) = scheduler::run_heartbeat_scheduler_tick(&pool_scheduler).await {
                    tracing::warn!("heartbeat scheduler tick: {}", e);
                }
            }
        });
        let state = routes::build_api_state(pool, &config);
        let mut app = Router::new().nest("/api", api_routes(state.clone()));
        if let Some(ref ui_dist) = config.ui_dist {
            tracing::info!("Serving UI from {}", ui_dist.display());
            app = app.fallback_service(static_fallback_service(ui_dist.clone()));
        } else {
            tracing::info!("UI_DIST not set; run from server-rs with ../ui/dist or set UI_DIST for UI");
        }
        let app = app
            .with_state(state)
            .layer(DefaultBodyLimit::max(config.http_body_max_bytes))
            .layer(CompressionLayer::new())
            .layer(SetRequestIdLayer::new(x_req_id.clone(), UuidRequestId))
            .layer(PropagateRequestIdLayer::new(x_req_id.clone()))
            .layer(cors_layer.clone())
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            ));
        tracing::info!("Paperclip (Rust) listening on http://{}", addr);
        axum::serve(listener, app.into_make_service()).await?;
    } else {
        tracing::warn!("DATABASE_URL not set: only /api/health is available; other list routes return 503");
        let api_router = Router::new().nest("/api", routes::api_routes_no_db());
        let app = if let Some(ref ui_dist) = config.ui_dist {
            tracing::info!("Serving UI from {}", ui_dist.display());
            api_router.merge(static_router(ui_dist.clone()))
        } else {
            api_router
        };
        let app = app
            .layer(DefaultBodyLimit::max(config.http_body_max_bytes))
            .layer(CompressionLayer::new())
            .layer(SetRequestIdLayer::new(x_req_id.clone(), UuidRequestId))
            .layer(PropagateRequestIdLayer::new(x_req_id))
            .layer(cors_layer)
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            ));
        tracing::info!("Paperclip (Rust) listening on http://{}", addr);
        axum::serve(listener, app.into_make_service()).await?;
    }
    Ok(())
}
