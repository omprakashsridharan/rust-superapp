use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::Serialize;
use std::time::Duration;
use tracing::{error, info};

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
    topic: String,
}

impl KafkaProducer {
    pub fn new(bootstrap_servers: String, topic: String) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");
        Self { producer, topic }
    }

    pub async fn produce(&self, key: impl ToBytes, payload: impl Serialize) -> bool {
        let serialized_data = serde_json::to_vec(&payload).expect("Failed to serialize payload");

        let record = FutureRecord::to(&self.topic)
            .payload(&serialized_data)
            .key(&key);
        let delivery_status = self.producer.send(record, Duration::from_secs(5)).await;
        if delivery_status.is_err() {
            error!("{}", delivery_status.err().unwrap().0.to_string());
            return false;
        } else {
            info!("message delivered");
            return true;
        }
    }
}
