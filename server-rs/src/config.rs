use std::collections::HashMap;
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
    /// Max request body size in bytes (env: HTTP_BODY_MAX_BYTES). 0 = use default 2 MiB.
    pub http_body_max_bytes: usize,
    /// Max concurrent adapter runs; 0 = unlimited (env: RUNNER_MAX_CONCURRENT_RUNS).
    pub runner_max_concurrent_runs: usize,
    /// Cap for HTTP adapter timeout in ms (env: RUNNER_HTTP_MAX_TIMEOUT_MS).
    pub runner_http_max_timeout_ms: u64,
    /// Cap for process adapter timeout in seconds (env: RUNNER_PROCESS_MAX_TIMEOUT_SECS).
    pub runner_process_max_timeout_secs: u64,
    /// CORS allowed origins; empty = allow any (env: CORS_ORIGINS, comma-separated).
    pub cors_origins: Vec<String>,
}

impl Config {
    /// Load optional config file (env CONFIG_FILE = path to JSON). Keys = env names, values = string or number.
    /// Env vars override file. Returns empty map if CONFIG_FILE unset or parse fails.
    fn load_config_file() -> HashMap<String, String> {
        let path = match env::var("CONFIG_FILE") {
            Ok(p) => p,
            Err(_) => return HashMap::new(),
        };
        let s = match std::fs::read_to_string(&path) {
            Ok(x) => x,
            Err(_) => return HashMap::new(),
        };
        let v: serde_json::Value = match serde_json::from_str(&s) {
            Ok(x) => x,
            Err(_) => return HashMap::new(),
        };
        let obj = match v.as_object() {
            Some(o) => o,
            None => return HashMap::new(),
        };
        let mut out = HashMap::new();
        for (k, val) in obj {
            let s = match val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => continue,
            };
            out.insert(k.clone(), s);
        }
        out
    }

    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        let file_vars = Self::load_config_file();
        let get = |key: &str| -> Option<String> {
            env::var(key).ok().or_else(|| file_vars.get(key).cloned())
        };

        let host = get("HOST").unwrap_or_else(|| "127.0.0.1".to_string());
        let port = get("PORT")
            .and_then(|s| s.parse().ok())
            .unwrap_or(3100);

        let database_url = get("DATABASE_URL");

        let ui_dist = get("UI_DIST")
            .map(PathBuf::from)
            .or_else(|| {
                let cwd = env::current_dir().ok()?;
                let candidate = cwd.join("../ui/dist");
                if candidate.join("index.html").exists() {
                    Some(candidate.canonicalize().ok().unwrap_or(candidate))
                } else {
                    None
                }
            })
            .filter(|p| p.join("index.html").exists());

        let db_pool_max_size = get("DB_POOL_MAX_SIZE")
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let db_pool_acquire_timeout_secs = get("DB_POOL_ACQUIRE_TIMEOUT_SECS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);
        let db_pool_idle_timeout_secs = get("DB_POOL_IDLE_TIMEOUT_SECS").and_then(|s| s.parse().ok());
        let scheduler_interval_secs = get("SCHEDULER_INTERVAL_SECS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        let http_body_max_bytes = get("HTTP_BODY_MAX_BYTES")
            .and_then(|s| s.parse().ok())
            .unwrap_or(2 * 1024 * 1024); // 2 MiB
        let runner_max_concurrent_runs = get("RUNNER_MAX_CONCURRENT_RUNS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let runner_http_max_timeout_ms = get("RUNNER_HTTP_MAX_TIMEOUT_MS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(300_000); // 5 min
        let runner_process_max_timeout_secs = get("RUNNER_PROCESS_MAX_TIMEOUT_SECS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(86400); // 24 h

        let cors_origins = get("CORS_ORIGINS")
            .map(|s| {
                s.split(',')
                    .map(str::trim)
                    .filter(|x| !x.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Self {
            database_url,
            host,
            port,
            ui_dist,
            db_pool_max_size,
            db_pool_acquire_timeout_secs,
            db_pool_idle_timeout_secs,
            scheduler_interval_secs,
            http_body_max_bytes,
            runner_max_concurrent_runs,
            runner_http_max_timeout_ms,
            runner_process_max_timeout_secs,
            cors_origins,
        }
    }
}
