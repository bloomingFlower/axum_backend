//! This module create database pool and manage the database connection

mod base;
mod error;
mod store;
pub mod task;

pub mod user;

pub use self::error::{Error, Result};
// Database Field
use crate::model::psql::store::{new_db_pool, Db};
use std::sync::Arc;

#[derive(Clone)]
pub struct ModelManager {
    db: Arc<Db>,
}

impl ModelManager {
    /// Create Constructor
    pub async fn new() -> Result<Self> {
        let db = Arc::new(new_db_pool().await?);
        Ok(ModelManager { db })
    }

    /// Return the Db pool reference
    /// (Only for the internal use of the model module)
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
