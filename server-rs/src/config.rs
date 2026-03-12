use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub database_url: Option<String>,
    pub host: String,
    pub port: u16,
    /// Directory containing built UI (index.html + assets). If set, served at /.
    pub ui_dist: Option<PathBuf>,
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

        Self {
            database_url,
            host,
            port,
            ui_dist,
        }
    }
}
