mod hn;

use crate::hn::HNSearchResult;
use rdkafka::config::ClientConfig;
use rdkafka::error::{KafkaError, KafkaResult};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info};

fn create_producer(host: &str) -> KafkaResult<FutureProducer> {
    ClientConfig::new().set("bootstrap.servers", host).create()
}

async fn send_to_kafka(
    host: &str,
    topic: &str,
    payload: Vec<HNSearchResult>,
) -> Result<(), KafkaError> {
    let producer = create_producer(host)?;

    for search_result in payload {
        let buffer = match serde_json::to_string(&search_result) {
            Ok(b) => b,
            Err(e) => {
                error!("Serialization error: {:?}", e);
                continue;
            }
        };
        let delivery_status = producer
            .send(
                FutureRecord::to(topic)
                    .payload(&buffer)
                    // key is optional, but it helps Kafka to keep messages with the same key in the same partition
                    .key("Rust"),
                Duration::from_secs(10),
            )
            .await;

        match delivery_status {
            Ok(delivery) => debug!("Sent: {:?}", delivery),
            Err((err, _)) => error!("Error: {:?}", err),
        }
    }

    Ok(())
}

pub async fn produce() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch Hacker News stories every 60 seconds
    let mut interval = interval(Duration::from_secs(60));
    let max_iterations = 10;

    for _ in 0..max_iterations {
        interval.tick().await;
        let stories = hn::fetch_hn_stories("Rust".into(), 100).await?;
        info!("Fetched {} stories", stories.hits.len());
        send_to_kafka("localhost:9092", "hnstories", stories.hits).await?;
    }

    Ok(())
}
