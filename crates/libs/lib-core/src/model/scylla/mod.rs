mod base;
mod error;
pub mod hnstory;
mod result;

use crate::config::core_config;
use crate::model::scylla::result::Result;
use scylla::{Session, SessionBuilder};
use tracing::info;

/// Create a new database connection.
/// Connection pool is automatically created by the ScyllaDB driver.
pub async fn db_conn() -> Result<Session> {
    let username = &core_config().SCYLLA_DB_USERNAME;
    let password = &core_config().SCYLLA_DB_PASSWORD;
    let uri = &core_config().SCYLLA_DB_URL;
    info!(
        "--> Scylla: Connecting to scylla db at {} with username {}",
        uri, username
    );
    // Create a ScyllaDB session with authentication
    let session = SessionBuilder::new()
        .known_node(uri)
        .user(username, password)
        .build()
        .await?;

    // Initialize the database
    hnstory::initialize(&session).await?;

    info!("--> Scylla: Connected to scylla db");

    Ok(session)
}
