mod config;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::message::Headers; // for the `next` method
use rdkafka::metadata::Metadata;
use rdkafka::Message;
use std::time::Duration;
use tracing::{debug, error, info};

use lib_core::model::scylla::db_conn;
use lib_core::model::scylla::hnstory::{add_hnstory, HNStory};

pub fn create_consumer() -> KafkaResult<StreamConsumer> {
    ClientConfig::new()
        .set(
            "bootstrap.servers",
            &config::consume_config().BOOTSTRAP_SERVER_URL,
        )
        .set("group.id", "hnstories_group")
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "true")
        .set("socket.timeout.ms", "4000")
        .set("auto.offset.reset", "earliest")
        // .set_log_level(RDKafkaLogLevel::Debug)
        // .set("debug", "all")
        .create()
}

pub async fn consume() {
    info!("Start Kafka consume");
    let consumer: StreamConsumer = create_consumer().expect("Consumer creation failed");

    consumer
        .subscribe(&["hnstories"])
        .expect("Can't subscribe to specified topics");

    let subscription = consumer.subscription().expect("Failed to get subscription");
    info!("Subscribed to the following topics:");
    for topic in subscription.elements() {
        println!("  - {}", topic.topic());
    }

    // Create a ScyllaDB session
    let session = db_conn().await.expect("Failed to create ScyllaDB session");

    loop {
        match consumer.recv().await {
            Err(e) => error!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        error!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                debug!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                         m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if !payload.is_empty() {
                    debug!("payload: {}", payload);
                    match serde_json::from_str::<HNStory>(&payload) {
                        Ok(hnstory) => {
                            if let Err(e) = add_hnstory(&session, hnstory).await {
                                error!("Failed to add hnstory: {}", e);
                            }
                        }
                        Err(e) => error!("Failed to parse hnstory from payload: {}", e),
                    }
                }

                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        info!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

pub async fn list_topics() -> KafkaResult<()> {
    let consumer: StreamConsumer = create_consumer()?;

    let metadata: Metadata = consumer.fetch_metadata(None, Some(Duration::from_secs(5)))?;

    info!("list of topic size: {}", metadata.topics().len());

    for topic in metadata.topics() {
        info!("topic name: {}", topic.name());
    }

    consume().await;

    Ok(())
}
