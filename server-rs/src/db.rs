use crate::config::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub async fn create_pool(database_url: &str, config: &Config) -> Result<PgPool, sqlx::Error> {
    let mut opts = PgPoolOptions::new()
        .max_connections(config.db_pool_max_size)
        .acquire_timeout(Duration::from_secs(config.db_pool_acquire_timeout_secs));
    if let Some(secs) = config.db_pool_idle_timeout_secs {
        opts = opts.idle_timeout(Duration::from_secs(secs));
    }
    opts.connect(database_url).await
}
