use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::Serialize;
use std::time::Duration;
use tracing::{error, info};

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(bootstrap_servers: String) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");
        Self { producer }
    }

    pub async fn produce(&self, topic: &str, key: impl ToBytes, payload: impl Serialize) -> bool {
        let serialized_data = serde_json::to_vec(&payload).expect("Failed to serialize payload");

        let record = FutureRecord::to(topic).payload(&serialized_data).key(&key);
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
