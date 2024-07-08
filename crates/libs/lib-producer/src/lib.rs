mod hn;
pub mod token;

use rdkafka::config::ClientConfig;
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::error::{KafkaError, KafkaResult};
use rdkafka::producer::{FutureProducer, FutureRecord};

use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info};

fn create_producer(host: &str) -> KafkaResult<FutureProducer> {
    ClientConfig::new().set("bootstrap.servers", host).create()
}

async fn send_to_kafka<T: serde::Serialize>(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    payload: &T,
) -> Result<(), KafkaError> {
    let buffer = serde_json::to_string(payload).map_err(|e| {
        error!("Serialization error: {:?}", e);
        KafkaError::MessageProduction(RDKafkaErrorCode::BadMessage)
    })?;

    let delivery_status = producer
        .send(
            FutureRecord::to(topic).payload(&buffer).key(key),
            Duration::from_secs(10),
        )
        .await;

    match delivery_status {
        Ok(delivery) => debug!("Sent: {:?}", delivery),
        Err((err, _)) => error!("Error: {:?}", err),
    }

    Ok(())
}

pub async fn produce() -> Result<(), Box<dyn std::error::Error>> {
    let producer = create_producer("localhost:9092")?;

    // HN stories task
    let hn_producer = producer.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            match hn::fetch_hn_stories("Rust".into(), 10).await {
                Ok(stories) => {
                    info!("Fetched {} stories", stories.hits.len());
                    for story in stories.hits {
                        if let Err(e) =
                            send_to_kafka(&hn_producer, "hnstories", "Rust", &story).await
                        {
                            error!("Failed to send HN story: {:?}", e);
                        }
                    }
                }
                Err(e) => error!("Failed to fetch HN stories: {:?}", e),
            }
        }
    });

    Ok(())
}

pub async fn produce_bitcoin_info() -> Result<(), Box<dyn std::error::Error>> {
    let producer = create_producer("localhost:9092")?;

    //Bitcoin info task
    let bitcoin_producer = producer.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(3600 * 24 / 30)); // 하루에 30번 호출 (약 48분마다)
        loop {
            interval.tick().await;
            match token::fetch_bitcoin_info().await {
                Ok(bitcoin_info) => {
                    if let Err(e) =
                        send_to_kafka(&bitcoin_producer, "token", "Bitcoin", &bitcoin_info).await
                    {
                        error!("Failed to send Bitcoin info: {:?}", e);
                    }
                }
                Err(e) => error!("Failed to fetch Bitcoin info: {:?}", e),
            }
        }
    });

    Ok(())
}
