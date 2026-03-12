mod auth;
mod config;
mod db;
mod models;
mod routes;
mod runner;
mod scheduler;

use axum::Router;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use routes::api_routes;

fn static_router(ui_dist: PathBuf) -> Router {
    let index_path = ui_dist.join("index.html");
    Router::new().nest_service(
        "/",
        ServeDir::new(&ui_dist).fallback(ServeFile::new(index_path)),
    )
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

    let api_router = if let Some(ref database_url) = config.database_url {
        let pool = db::create_pool(database_url).await?;
        tracing::info!("PostgreSQL connected");
        let pool_scheduler = pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = scheduler::run_heartbeat_scheduler_tick(&pool_scheduler).await {
                    tracing::warn!("heartbeat scheduler tick: {}", e);
                }
            }
        });
        Router::new().nest("/api", api_routes(pool))
    } else {
        tracing::warn!("DATABASE_URL not set: only /api/health is available; other list routes return 503");
        Router::new().nest("/api", routes::api_routes_no_db())
    };

    let app = if let Some(ref ui_dist) = config.ui_dist {
        tracing::info!("Serving UI from {}", ui_dist.display());
        api_router.merge(static_router(ui_dist.clone()))
    } else {
        tracing::info!("UI_DIST not set; run from server-rs with ../ui/dist or set UI_DIST for UI");
        api_router
    };

    let app = app.layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    let addr = SocketAddr::from((config.host.as_str().parse::<std::net::IpAddr>()?, config.port));
    tracing::info!("Paperclip (Rust) listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
