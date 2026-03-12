use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub database_url: Option<String>,
    pub host: String,
    pub port: u16,
    /// Directory containing built UI (index.html + assets). If set, served at /.
    pub ui_dist: Option<PathBuf>,
    /// DB pool max connections (env: DB_POOL_MAX_SIZE).
    pub db_pool_max_size: u32,
    /// DB pool acquire timeout in seconds (env: DB_POOL_ACQUIRE_TIMEOUT_SECS).
    pub db_pool_acquire_timeout_secs: u64,
    /// DB pool idle timeout in seconds; None = use driver default (env: DB_POOL_IDLE_TIMEOUT_SECS).
    pub db_pool_idle_timeout_secs: Option<u64>,
    /// Heartbeat scheduler tick interval in seconds (env: SCHEDULER_INTERVAL_SECS).
    pub scheduler_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3100);

        let database_url = env::var("DATABASE_URL").ok();

        let ui_dist = env::var("UI_DIST")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                // Default: ../ui/dist when running from server-rs
                let cwd = env::current_dir().ok()?;
                let candidate = cwd.join("../ui/dist");
                if candidate.join("index.html").exists() {
                    Some(candidate.canonicalize().ok().unwrap_or(candidate))
                } else {
                    None
                }
            })
            .filter(|p| p.join("index.html").exists());

        let db_pool_max_size = env::var("DB_POOL_MAX_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let db_pool_acquire_timeout_secs = env::var("DB_POOL_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);
        let db_pool_idle_timeout_secs = env::var("DB_POOL_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok());
        let scheduler_interval_secs = env::var("SCHEDULER_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        Self {
            database_url,
            host,
            port,
            ui_dist,
            db_pool_max_size,
            db_pool_acquire_timeout_secs,
            db_pool_idle_timeout_secs,
            scheduler_interval_secs,
        }
    }
}
