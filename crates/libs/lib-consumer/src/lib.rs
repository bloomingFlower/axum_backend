mod config;

use lib_core::model::scylla::db_conn;
use lib_core::model::scylla::hnstory::add_hnstory;
use lib_core::model::scylla::hnstory::HNStory;
use lib_producer::token::BitcoinInfo;
use rdkafka::error::KafkaError;
use tokio::time::{timeout, Duration};

use std::pin::Pin;

use futures::stream::Stream;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::Consumer;
use rdkafka::consumer::{CommitMode, StreamConsumer};
use rdkafka::error::KafkaResult;
use rdkafka::message::Headers; // for the `next` method
use rdkafka::message::OwnedMessage;
use rdkafka::metadata::Metadata;
use rdkafka::Message;
use rdkafka::Offset;

use tracing::{debug, error, info};

// Function to create a Kafka consumer
pub fn create_consumer() -> KafkaResult<StreamConsumer> {
    // Create a new Kafka client configuration
    ClientConfig::new()
        .set(
            "bootstrap.servers",
            &config::consume_config().KAFKA_BOOTSTRAP_SERVERS,
        )
        .set("group.id", "unique_consumer_group_id")
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "true")
        .set("socket.timeout.ms", "4000")
        .set("auto.offset.reset", "latest")
        .set("fetch.min.bytes", "1")
        .set("session.timeout.ms", "60000")
        .set("heartbeat.interval.ms", "20000")
        .set("max.poll.interval.ms", "600000")
        .set("reconnect.backoff.max.ms", "30000")
        .set("reconnect.backoff.ms", "2000")
        .set("allow.auto.create.topics", "true")
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
    let consumer: StreamConsumer =
        create_consumer().expect("--> Kafka Consumer: Consumer creation failed");

    // Define the topics to subscribe to
    let topics = vec![topic_name];

    // Subscribe to the specified topics
    consumer
        .subscribe(topics.as_slice())
        .expect("--> Kafka Consumer: Can't subscribe to specified topics");

    // Get the subscription details
    let subscription = consumer
        .subscription()
        .expect("--> Kafka Consumer: Failed to get subscription");
    info!("--> Kafka Consumer: Subscribed to the following topics:");
    for topic in subscription.elements() {
        println!("  - {}", topic.topic());
    }

    // Sleep for 2 seconds to allow the consumer to start consuming messages
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Create a ScyllaDB session
    let session = db_conn()
        .await
        .expect("--> Kafka Consumer: Failed to create ScyllaDB session");

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
                        error!(
                            "--> Kafka Consumer: Error while deserializing message payload: {:?}",
                            e
                        );
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
) -> KafkaResult<Pin<Box<dyn Stream<Item = KafkaResult<OwnedMessage>> + Send>>> {
    // Create a Kafka consumer
    let consumer: StreamConsumer = create_consumer()?;
    // Subscribe to the specified topic
    consumer.subscribe(&[topic_name])?;

    // Sleep for 2 seconds to allow the consumer to start consuming messages
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Create a pinned boxed stream from the consumer
    // Pin<Box<dyn ...>> is used to create a pinned, heap-allocated stream
    // that can be moved across await points in async code
    let stream = Box::pin(async_stream::stream! {
        loop {
            // Receive a message from the consumer
            match consumer.recv().await {
                Ok(msg) => yield Ok(msg.detach()),
                Err(e) => yield Err(e),
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

    info!(
        "--> Kafka Consumer: list of topic size: {}",
        metadata.topics().len()
    );
    for topic in metadata.topics() {
        info!("--> Kafka Consumer: topic name: {}", topic.name());
    }

    // Consume messages from the "hnstories" topic
    consume("hnstories").await;

    // Return Ok result
    Ok(())
}

pub async fn consume_latest_message(topic: &str) -> Result<Option<BitcoinInfo>, KafkaError> {
    let consumer: StreamConsumer = create_consumer()?;
    consumer.subscribe(&[topic])?;

    info!(
        "--> Kafka Consumer: Consuming latest message from topic: {}",
        topic
    );

    // 메타데이터 가져오기
    let metadata = consumer.fetch_metadata(Some(topic), Duration::from_secs(10))?;
    let partitions = metadata
        .topics()
        .first()
        .map(|t| t.partitions().len())
        .unwrap_or(0);

    for partition in 0..partitions {
        match consumer.fetch_watermarks(topic, partition as i32, Duration::from_secs(10)) {
            Ok((_, high_watermark)) => {
                info!(
                    "--> Kafka Consumer: Partition {}, High watermark: {}",
                    partition, high_watermark
                );
                if high_watermark > 0 {
                    // 마지막 오프셋으로 이동
                    consumer.seek(
                        topic,
                        partition as i32,
                        Offset::Offset(high_watermark - 1),
                        Duration::from_secs(10),
                    )?;

                    // 메시지 수신 시도
                    match tokio::time::timeout(Duration::from_secs(5), consumer.recv()).await {
                        Ok(Ok(message)) => {
                            if let Some(payload) = message.payload() {
                                match serde_json::from_slice::<BitcoinInfo>(payload) {
                                    Ok(bitcoin_info) => return Ok(Some(bitcoin_info)),
                                    Err(e) => error!(
                                        "--> Kafka Consumer: Failed to deserialize message: {:?}",
                                        e
                                    ),
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!("--> Kafka Consumer: Failed to receive message: {:?}", e)
                        }
                        Err(_) => error!("--> Kafka Consumer: Timeout while receiving message"),
                    }
                }
            }
            Err(e) => error!(
                "--> Kafka Consumer: Failed to fetch watermarks for partition {}: {:?}",
                partition, e
            ),
        }
    }

    info!("--> Kafka Consumer: No valid messages found");
    Ok(None)
}

// async fn try_consume_latest_message(topic: &str) -> Result<Option<BitcoinInfo>, KafkaError> {
//     // 컨슈머 생성 및 토픽 구독
//     let consumer: StreamConsumer = create_consumer()?;
//     consumer.subscribe(&[topic])?;

//     let timeout = Duration::from_secs(60);
//     let mut retry_count = 0;
//     let max_retries = 5;

//     // 메타데이터 가져오기
//     let metadata = consumer.fetch_metadata(Some(topic), timeout)?;
//     let partitions = metadata
//         .topics()
//         .first()
//         .map(|t| t.partitions().len())
//         .unwrap_or(0);

//     while retry_count < max_retries {
//         info!(
//             "--> Kafka Consumer: Attempt {} of {}",
//             retry_count + 1,
//             max_retries
//         );

//         // 모든 파티션에 대해 시도
//         for partition in 0..partitions {
//             match consumer.fetch_watermarks(topic, partition as i32, timeout) {
//                 Ok((_, high_watermark)) => {
//                     info!(
//                         "--> Kafka Consumer: Partition {}, High watermark: {}",
//                         partition, high_watermark
//                     );
//                     if high_watermark > 0 {
//                         // 마지막 오프셋으로 이동
//                         if let Err(e) = consumer.seek(
//                             topic,
//                             partition as i32,
//                             Offset::Offset(high_watermark - 1),
//                             timeout,
//                         ) {
//                             error!("--> Kafka Consumer: Failed to seek to offset {} in partition {}: {:?}", high_watermark - 1, partition, e);
//                             continue;
//                         }

//                         // 메시지 수신 시도
//                         match tokio::time::timeout(timeout, consumer.recv()).await {
//                             Ok(Ok(message)) => {
//                                 if let Some(payload) = message.payload() {
//                                     match serde_json::from_slice::<BitcoinInfo>(payload) {
//                                         Ok(bitcoin_info) => return Ok(Some(bitcoin_info)),
//                                         Err(e) => error!("--> Kafka Consumer: Failed to deserialize message: {:?}", e),
//                                     }
//                                 }
//                             }
//                             Ok(Err(e)) => {
//                                 error!("--> Kafka Consumer: Failed to receive message: {:?}", e)
//                             }
//                             Err(_) => error!("--> Kafka Consumer: Timeout while receiving message"),
//                         }
//                     }
//                 }
//                 Err(e) => error!(
//                     "--> Kafka Consumer: Failed to fetch watermarks for partition {}: {:?}",
//                     partition, e
//                 ),
//             }
//         }

//         retry_count += 1;
//         if retry_count < max_retries {
//             info!("--> Kafka Consumer: Retrying in 1 second...");
//             tokio::time::sleep(Duration::from_secs(1)).await;
//         }
//     }

//     info!(
//         "--> Kafka Consumer: No valid messages found after {} attempts",
//         max_retries
//     );
//     Ok(None)
// }
