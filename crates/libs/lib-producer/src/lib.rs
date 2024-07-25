mod config;
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
    info!("--> Kafka Producer: Creating producer");
    ClientConfig::new().set("bootstrap.servers", host).create()
}

async fn send_to_kafka<T: serde::Serialize>(
    producer: &FutureProducer,
    topic: &str,
    key: &str,
    payload: &T,
) -> Result<(), KafkaError> {
    let buffer = serde_json::to_string(payload).map_err(|e| {
        error!("--> Kafka Producer: Serialization error: {:?}", e);
        KafkaError::MessageProduction(RDKafkaErrorCode::BadMessage)
    })?;

    let delivery_status = producer
        .send(
            FutureRecord::to(topic).payload(&buffer).key(key),
            Duration::from_secs(10),
        )
        .await;

    match delivery_status {
        Ok(delivery) => debug!("--> Kafka Producer: Sent: {:?}", delivery),
        Err((err, _)) => error!("--> Kafka Producer: Error: {:?}", err),
    }

    Ok(())
}

pub async fn produce() -> Result<(), Box<dyn std::error::Error>> {
    let producer = create_producer("")?;

    // HN stories task
    let hn_producer = producer.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            match hn::fetch_hn_stories("Rust".into(), 10).await {
                Ok(stories) => {
                    info!("--> Kafka Producer: Fetched {} stories", stories.hits.len());
                    for story in stories.hits {
                        if let Err(e) =
                            send_to_kafka(&hn_producer, "hnstories", "Rust", &story).await
                        {
                            error!("--> Kafka Producer: Failed to send HN story: {:?}", e);
                        }
                    }
                }
                Err(e) => error!("--> Kafka Producer: Failed to fetch HN stories: {:?}", e),
            }
        }
    });

    Ok(())
}

pub async fn produce_bitcoin_info() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let producer = create_producer(&config::producer_config().KAFKA_BOOTSTRAP_SERVERS)?;
    info!("--> Kafka Info Producer: Producer created");
    let bitcoin_producer = producer.clone();
    tokio::spawn(async move {
        info!("--> Kafka Info Producer: Spawned");
        let mut interval = interval(Duration::from_secs(3600 * 24 / 30));
        loop {
            interval.tick().await;
            let max_retries = 3;
            let mut retry_count = 0;
            while retry_count < max_retries {
                match fetch_and_send_bitcoin_info(&bitcoin_producer).await {
                    Ok(_) => break,
                    Err(e) => {
                        error!(
                            "--> Kafka Info Producer: Attempt {} failed: {:?}",
                            retry_count + 1,
                            e
                        );
                        retry_count += 1;
                        if retry_count < max_retries {
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
            if retry_count == max_retries {
                error!(
                    "--> Kafka Info Producer: Failed after {} attempts",
                    max_retries
                );
            }
        }
    });

    Ok(())
}

async fn fetch_and_send_bitcoin_info(
    producer: &FutureProducer,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bitcoin_info = token::fetch_bitcoin_info().await?;
    info!("--> Kafka Info Producer: Fetched Bitcoin info");
    send_to_kafka(producer, "token", "Bitcoin", &bitcoin_info).await?;
    Ok(())
}
