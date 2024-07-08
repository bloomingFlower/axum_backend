mod config;

use lib_core::model::scylla::db_conn;
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
use std::time::Duration;
use tracing::{debug, error, info};

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

pub async fn consume(topic_name: &str) {
    info!("Start Kafka consume for topics: {:?}", topic_name);
    let consumer: StreamConsumer = create_consumer().expect("Consumer creation failed");

    let topics = vec![topic_name];

    consumer
        .subscribe(topics.as_slice())
        .expect("Can't subscribe to specified topics");

    let subscription = consumer.subscription().expect("Failed to get subscription");
    info!("Subscribed to the following topics:");
    for topic in subscription.elements() {
        println!("  - {}", topic.topic());
    }

    // Create a ScyllaDB session
    db_conn().await.expect("Failed to create ScyllaDB session");

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
                // if !payload.is_empty() {
                //     match serde_json::from_str::<HNStory>(&payload) {
                //         Ok(hnstory) => {
                //             if let Err(e) = add_hnstory(&session, hnstory).await {
                //                 error!("Failed to add {}: {}", m.topic(), e);
                //             }
                //         }
                //         Err(e) => error!("Failed to parse hnstory from payload: {}", e),
                //     }
                // }

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

pub async fn consume_stream(
    topic_name: &str,
) -> KafkaResult<Pin<Box<dyn Stream<Item = KafkaResult<OwnedMessage>> + Send>>> {
    let consumer: StreamConsumer = create_consumer()?;
    consumer.subscribe(&[topic_name])?;

    // Move consumer into the stream
    let stream = Box::pin(async_stream::stream! {
        loop {
            match consumer.recv().await {
                Ok(msg) => yield Ok(msg.detach()),
                Err(e) => yield Err(e),
            }
        }
    });
    Ok(stream)
}

pub async fn list_topics() -> KafkaResult<()> {
    let consumer: StreamConsumer = create_consumer()?;

    let metadata: Metadata = consumer.fetch_metadata(None, Some(Duration::from_secs(5)))?;

    info!("list of topic size: {}", metadata.topics().len());

    for topic in metadata.topics() {
        info!("topic name: {}", topic.name());
    }

    consume("hnstories").await;

    Ok(())
}
