mod auth;
mod config;
mod db;
mod models;
mod routes;
mod runner;
mod scheduler;

use axum::extract::DefaultBodyLimit;
use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
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

    let layers = (
        DefaultBodyLimit::max(config.http_body_max_bytes),
        CompressionLayer::new(),
        CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any),
    );

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
            .layer(layers.0)
            .layer(layers.1)
            .layer(layers.2);
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
            .layer(layers.0)
            .layer(layers.1)
            .layer(layers.2);
        tracing::info!("Paperclip (Rust) listening on http://{}", addr);
        axum::serve(listener, app.into_make_service()).await?;
    }
    Ok(())
}
