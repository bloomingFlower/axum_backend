use std::time::Duration;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::Message;
use rdkafka::error::KafkaResult;
use rdkafka::message::Headers; // for the `next` method
use rdkafka::metadata::Metadata;

pub async fn consume() {
    println!("Start Kafka consume");
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "hnstories_group")
        .set("enable.partition.eof", "false")
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "latest")
        .set_log_level(RDKafkaLogLevel::Debug)
        .set("debug", "all") 
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["hnstories"])
        .expect("Can't subscribe to specified topics");

    // Print the list of topics the consumer is subscribed to
    let subscription = consumer.subscription().expect("Failed to get subscription");
    println!("Subscribed to the following topics:");
    for topic in subscription.elements() {
        println!("  - {}", topic.topic());
    }
    
    loop {
        match consumer.recv().await {
            Err(e) => println!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        println!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

pub async fn list_topics() -> KafkaResult<()> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "hnstories_group")
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "1000")
        .set("session.timeout.ms", "30000")
        .create()?;

    let metadata: Metadata = consumer.fetch_metadata(None, Some(Duration::from_secs(5)))?;

    println!("list of topic size: {}", metadata.topics().len());

    for topic in metadata.topics() {
        println!("topic name: {}", topic.name());
    }

    consume().await;

    Ok(())
}