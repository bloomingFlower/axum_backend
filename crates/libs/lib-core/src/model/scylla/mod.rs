mod base;
mod error;
pub mod hnstory;
mod result;

use crate::model::scylla::result::Result;
use scylla::{Session, SessionBuilder};

/// Create a new database connection.
/// Connection pool is automatically created by the ScyllaDB driver.
pub async fn db_conn() -> Result<(Session)> {
    let username = std::env::var("SCYLLA_USERNAME").expect("SCYLLA_USERNAME must be set");
    let password = std::env::var("SCYLLA_PASSWORD").expect("SCYLLA_PASSWORD must be set");
    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    println!("connecting to db at {} with username {}", uri, username);
    // Create a ScyllaDB session with authentication
    let session = SessionBuilder::new()
        .known_node(&uri)
        .user(username, password)
        .build()
        .await?;

    hnstory::initialize(&session).await?;

    Ok(session)
}
