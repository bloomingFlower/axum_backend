use scylla::SessionBuilder;
use uuid::Uuid;

use crate::duration::Duration;
use crate::result::Result;
use crate::temperature_measurement::TemperatureMeasurement;

mod db;
mod duration;
mod result;
mod temperature_measurement;

#[tokio::main]
async fn main() -> Result<()> {
    let username = std::env::var("SCYLLA_USERNAME").expect("SCYLLA_USERNAME must be set");
    let password = std::env::var("SCYLLA_PASSWORD").expect("SCYLLA_PASSWORD must be set");
    let uri = std::env::var("SCYLLA_DB_URL").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    println!("connecting to db at {} with username {}", uri, username);
    // Create a ScyllaDB session with authentication
    let session = SessionBuilder::new()
        .known_node(&uri)
        .user(username, password)
        .build()
        .await?;

    db::initialize(&session).await?;

    println!("Adding measurements");
    let measurement = TemperatureMeasurement {
        device: Uuid::parse_str("72f6d49c-76ea-44b6-b1bb-9186704785db")?,
        time: Duration::seconds(1000000000001),
        temperature: 40,
    };
    db::add_measurement(&session, measurement).await?;

    let measurement = TemperatureMeasurement {
        device: Uuid::parse_str("72f6d49c-76ea-44b6-b1bb-9186704785db")?,
        time: Duration::seconds(1000000000003),
        temperature: 60,
    };
    db::add_measurement(&session, measurement).await?;

    println!("Selecting measurements");
    let measurements = db::select_measurements(
        &session,
        Uuid::parse_str("72f6d49c-76ea-44b6-b1bb-9186704785db")?,
        Duration::seconds(1000000000000),
        Duration::seconds(10000000000009),
    )
    .await?;
    println!("     >> Measurements: {:?}", measurements);

    Ok(())
}
