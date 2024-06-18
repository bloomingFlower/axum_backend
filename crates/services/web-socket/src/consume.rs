use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::config::ClientConfig;
use rdkafka::Message;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyMessage {
    key: String,
    value: String,
}

pub async fn consume(tx: broadcast::Sender<MyMessage>) {
    let consumer: BaseConsumer = ClientConfig::new()
        .set("group_id", "ws_group")
        .set("bootstrap.servers", "localhost:9092")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("WS Consumer creation failed");
    
    consumer.subscribe(&["ws_topic"]).expect("WS Subscription failed");
    
    loop {
        match consumer.poll(None) {
            Some(Ok(msg)) => {
                if let Some(payload) = msg.payload() {
                    let my_msg: MyMessage = serde_json::from_slice(payload).unwrap();
                    tx.send(my_msg).unwrap();
                }
            },
            Some(Err(e)) => eprintln!("Kafka Error: {}", e),
            None => (),
        }
    }
}