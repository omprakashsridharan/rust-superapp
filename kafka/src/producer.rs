use std::time::Duration;
use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(producer: FutureProducer) -> Self {
        Self {
            producer,
        }
    }

    pub async fn produce<K: ToBytes, V: ToBytes>(&self, topic: &str, key: K, payload: V) -> bool {
        let record = FutureRecord::to(topic).payload(&payload).key(&key);
        let delivery_status = self.producer.send(record, Duration::from_secs(0)).await;
        return delivery_status.is_ok();
    }
}