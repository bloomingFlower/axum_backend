use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::Message;
use futures::stream::StreamExt; // for the `next` method

pub async fn consume() {
    println!("Start Kafka consume");
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "hnstories_group")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["hnstories"])
        .expect("Can't subscribe to specified topics");

    let mut message_stream = consumer.stream();

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(message) => {
                let payload = match message.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(_)) => {
                        println!("Message payload is not a string");
                        continue;
                    }
                };
                println!("Message: {}", payload);
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            }
            Err(err) => println!("Error: {}", err),
        }
    }
}