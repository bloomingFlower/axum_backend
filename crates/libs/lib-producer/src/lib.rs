mod hn;

use crate::hn::HNSearchResult;
use rdkafka::config::ClientConfig;
use rdkafka::error::{KafkaError, KafkaResult};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

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
                println!("Serialization error: {:?}", e);
                continue;
            }
        };
        let delivery_status = producer
            .send(
                FutureRecord::to(topic).payload(&buffer).key("some_key"),
                Duration::from_secs(10),
            )
            .await;

        match delivery_status {
            Ok(delivery) => println!("Sent: {:?}", delivery),
            Err((err, _)) => println!("Error: {:?}", err),
        }
    }

    Ok(())
}

pub async fn produce() -> Result<(), Box<dyn std::error::Error>> {
    let stories = hn::fetch_hn_stories("Rust".into(), 100).await?;
    println!("Fetched {} stories", stories.hits.len());
    send_to_kafka("localhost:9092", "hnstories", stories.hits).await?;

    Ok(())
}
