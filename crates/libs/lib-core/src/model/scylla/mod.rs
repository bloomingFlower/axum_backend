mod base;
mod error;
pub mod hnstory;
mod result;

pub use self::error::{Error, Result};

use crate::config::core_config;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;
use tracing::debug;

/// Create a new database connection.
/// Connection pool is automatically created by the ScyllaDB driver.
pub async fn db_conn() -> Result<Session> {
    let username = &core_config().SCYLLA_DB_USERNAME;
    let password = &core_config().SCYLLA_DB_PASSWORD;
    let uri = &core_config().SCYLLA_DB_URL;
    debug!(
        "--> Scylla: Connecting to scylla db at {} with username {}",
        uri, username
    );
    // Create a ScyllaDB session with authentication
    let session = SessionBuilder::new()
        .known_node(uri)
        .user(username, password)
        .build()
        .await?;
    debug!("--> Scylla: Connected to scylla db");

    Ok(session)
}

pub struct ScyllaManager {
    session: Arc<Session>,
}

impl ScyllaManager {
    // Constructor
    pub async fn new() -> Result<Arc<Self>> {
        let session = Arc::new(db_conn().await?);
        Ok(Arc::new(ScyllaManager { session }))
    }

    // Return the Session reference
    pub fn session(&self) -> &Session {
        &self.session
    }
}

// Initialize the scylla database
pub async fn initialize(session: &Session) -> std::result::Result<(), Error> {
    hnstory::initialize(session)
        .await
        .map_err(|e| Error::ScyllaError(e.to_string()))?;
    Ok(())
}
