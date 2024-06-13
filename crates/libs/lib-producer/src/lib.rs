mod hn;

use crate::hn::HNSearchResult;
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaError;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

async fn send_to_kafka(
    host: &str,
    topic: &str,
    payload: Vec<HNSearchResult>,
) -> Result<(), KafkaError> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", host)
        .create()?;

    for search_result in payload {
        let buffer = serde_json::to_string(&search_result).unwrap();
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
    let stories = hn::fetch_hn_stories("Ruby".into(), 100).await?;
    println!("Fetched {} stories", stories.hits.len());
    send_to_kafka("localhost:9092", "hnstories", stories.hits)
        .await
        .expect("Kafka test failed!");

    Ok(())
}
