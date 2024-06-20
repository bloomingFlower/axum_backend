mod error;

pub use self::error::{Error, Result};
use std::time::Duration;

use crate::core_config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// Database Field
pub type Db = Pool<Postgres>;

/// Create a new database pool
pub async fn new_db_pool() -> Result<Db> {
    let max_connections = if cfg!(test) { 1 } else { 5 };

    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_millis(500))
        .connect(&core_config().PSQL_DB_URL)
        .await
        .map_err(|e| Error::FailToCreatePool(e.to_string()))
}
