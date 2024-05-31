mod error;

pub use self::error::{Error, Result};
use std::time::Duration;

use crate::config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Database Field
pub type Db = Pool<Postgres>;

/// Create a new database pool
pub async fn new_db_pool() -> Result<Db> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&config::load_config().DB_URL)
        .await
        .map_err(|e| Error::FailToCreatePool(e.to_string()))
}
