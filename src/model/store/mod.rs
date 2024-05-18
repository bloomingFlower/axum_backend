use crate::error;

pub use self::error::{Error, Result};

use crate::config;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_millis(500))
        .connect(&config::load_config().DB_URL)
        .await
        .map_err(|e| Error::FailToCreatePool(e.to_string()))
}
