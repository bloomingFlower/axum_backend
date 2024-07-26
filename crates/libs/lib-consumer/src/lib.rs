mod config;

use lib_core::model::scylla::db_conn;
use lib_core::model::scylla::hnstory::add_hnstory;
use lib_core::model::scylla::hnstory::HNStory;
use lib_producer::token::BitcoinInfo;

use std::pin::Pin;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::RwLock;

use futures::stream::Stream;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::{CommitMode, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::message::Headers; // for the `next` method
use rdkafka::message::{OwnedHeaders, OwnedMessage};
use rdkafka::metadata::Metadata;
use rdkafka::Message;
use rdkafka::Timestamp;

use tracing::{debug, error, info};

// Global cache to store the last received message
static LAST_MESSAGE: LazyLock<Arc<RwLock<Option<String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

// Function to update the cache with the latest message
async fn update_cache(message: String) {
    let mut cache = LAST_MESSAGE.write().await;
    *cache = Some(message);
}

// Function to get the cached message
pub async fn get_cached_message() -> Option<String> {
    let cache = LAST_MESSAGE.read().await;
    cache.clone()
}

// Function to create a Kafka consumer
pub fn create_consumer() -> KafkaResult<StreamConsumer> {
    // Create a new Kafka client configuration
    ClientConfig::new()
        .set(
            "bootstrap.servers",
            &config::consume_config().KAFKA_BOOTSTRAP_SERVERS,
        )
        .set("group.id", "consumer_group")
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "true")
        .set("socket.timeout.ms", "4000")
        .set("auto.offset.reset", "earliest")
        .set("fetch.min.bytes", "1")
        // .set_log_level(RDKafkaLogLevel::Debug)
        // .set("debug", "all")
        .create()
}

// Asynchronous function to consume messages from a Kafka topic
pub async fn consume(topic_name: &str) {
    info!(
        "--> Kafka Consumer: Start Kafka consume for topics: {:?}",
        topic_name
    );
    // Create a Kafka consumer
    let consumer: StreamConsumer = create_consumer().expect("Consumer creation failed");

    // Define the topics to subscribe to
    let topics = vec![topic_name];

    // Subscribe to the specified topics
    consumer
        .subscribe(topics.as_slice())
        .expect("Can't subscribe to specified topics");

    // Get the subscription details
    let subscription = consumer.subscription().expect("Failed to get subscription");
    info!("--> Kafka Consumer: Subscribed to the following topics:");
    for topic in subscription.elements() {
        println!("  - {}", topic.topic());
    }

    // Sleep for 2 seconds to allow the consumer to start consuming messages
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Create a ScyllaDB session
    let session = db_conn().await.expect("Failed to create ScyllaDB session");

    // Infinite loop to continuously consume messages
    loop {
        // Receive a message from the consumer
        match consumer.recv().await {
            Err(e) => error!("--> Kafka Consumer: Kafka error: {}", e),
            // Process the received message
            Ok(m) => {
                // Extract the payload from the message
                let payload = match m.payload_view::<str>() {
                    // if payload is empty, return empty string
                    None => "",
                    // if payload is not empty, return the payload
                    Some(Ok(s)) => s,
                    // if payload is not empty, but error, return empty string
                    Some(Err(e)) => {
                        error!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                // Log the details of the received message
                debug!(
                    "Received message:\n\
                     Key: {:?}\n\
                     Payload: {}\n\
                     Topic: {}\n\
                     Partition: {}\n\
                     Offset: {}\n\
                     Timestamp: {:?}",
                    m.key(),
                    payload,
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );

                // if payload is not empty, deserialize the payload and add to ScyllaDB
                if !payload.is_empty() {
                    match serde_json::from_str::<HNStory>(payload) {
                        Ok(hnstory) => {
                            if let Err(e) = add_hnstory(&session, hnstory).await {
                                error!("--> Kafka Consumer: Failed to add {}: {}", m.topic(), e);
                            }
                        }
                        Err(e) => error!(
                            "--> Kafka Consumer: Failed to parse hnstory from payload: {}",
                            e
                        ),
                    }
                }

                // Update the cache with the new message
                // update_cache(payload.to_string()).await;
                // info!("Cache updated with new message: {}", payload);

                // Log the headers of the message if present
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        info!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                // Commit the message offset asynchronously
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

// Asynchronous function to consume messages from a Kafka topic as a stream
pub async fn consume_stream(
    topic_name: &str,
) -> KafkaResult<Pin<Box<dyn Stream<Item = KafkaResult<OwnedMessage>> + Send + '_>>> {
    let consumer: StreamConsumer = create_consumer()?;
    consumer.subscribe(&[topic_name])?;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let topic_name = Arc::new(topic_name.to_string());

    let stream = Box::pin(async_stream::stream! {
        let topic_name = topic_name.clone();
        loop {
            match consumer.recv().await {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        if let Ok(bitcoin_info) = serde_json::from_slice::<BitcoinInfo>(payload) {
                            update_cache(serde_json::to_string(&bitcoin_info).unwrap()).await;
                            info!("Cache updated with new message: {:?}", bitcoin_info);
                        }
                    }
                    yield Ok(msg.detach())
                },
                Err(e) => {
                    error!("Error receiving message: {:?}", e);
                    if let Some(cached_message) = get_cached_message().await {
                        if let Ok(bitcoin_info) = serde_json::from_str::<BitcoinInfo>(&cached_message) {
                            info!("Using cached Bitcoin info: {:?}", bitcoin_info);
                            let owned_message = OwnedMessage::new(
                                None, // key
                                Some(cached_message.into_bytes()), // payload
                                topic_name.to_string(), // topic
                                Timestamp::NotAvailable,
                                0, // partition
                                0, // offset
                                Some(OwnedHeaders::new()), // headers
                            );
                            yield Ok(owned_message);
                        } else {
                            yield Err(e);
                        }
                    } else {
                        yield Err(e);
                    }
                },
            }
        }
    });

    Ok(stream)
}

// Asynchronous function to list Kafka topics
pub async fn list_topics() -> KafkaResult<()> {
    // Create a Kafka consumer
    let consumer: StreamConsumer = create_consumer()?;

    // Fetch metadata for the topics
    let metadata: Metadata = consumer.fetch_metadata(None, Some(Duration::from_secs(5)))?;

    info!("list of topic size: {}", metadata.topics().len());
    for topic in metadata.topics() {
        info!("topic name: {}", topic.name());
    }

    // Consume messages from the "hnstories" topic
    consume("hnstories").await;

    // Return Ok result
    Ok(())
}
