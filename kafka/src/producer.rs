use rdkafka::message::ToBytes;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::Serialize;
use std::time::Duration;

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
        let delivery_status = self.producer.send(record, Duration::from_secs(0)).await;
        if delivery_status.is_err() {
            println!("{}", delivery_status.err().unwrap().0.to_string());
            return false;
        }

        return true;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;
    use rdkafka::{
        consumer::{Consumer, StreamConsumer},
        ClientConfig, Message,
    };
    use serde::{Deserialize, Serialize};
    use testcontainers::{clients, images::kafka};

    use super::KafkaProducer;

    #[tokio::test]
    async fn test_produce() {
        let docker = clients::Cli::default();
        let kafka_node = docker.run(kafka::Kafka::default());

        let bootstrap_servers = format!(
            "127.0.0.1:{}",
            kafka_node.get_host_port_ipv4(kafka::KAFKA_PORT)
        );

        let kafka_producer = KafkaProducer::new(bootstrap_servers.clone());

        let consumer = ClientConfig::new()
            .set("group.id", "testcontainer-rs")
            .set("bootstrap.servers", &bootstrap_servers)
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create::<StreamConsumer>()
            .expect("Failed to create Kafka StreamConsumer");
        let topic = "test-topic";
        let key = "test-key";
        let payload = "test-payload";
        let produce_result = kafka_producer
            .produce(topic, key.to_string(), payload.to_string())
            .await;
        assert!(produce_result);

        consumer
            .subscribe(&[topic])
            .expect("Failed to subscribe to the topic");

        let mut message_stream = consumer.stream();
        let borrowed_message = tokio::time::timeout(Duration::from_secs(10), message_stream.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            payload.to_string(),
            borrowed_message
                .unwrap()
                .payload_view::<str>()
                .unwrap()
                .unwrap()
        );
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    struct Custom {
        value: String,
    }

    #[tokio::test]
    async fn test_custom_struct_produce() {
        let docker = clients::Cli::default();
        let kafka_node = docker.run(kafka::Kafka::default());

        let bootstrap_servers = format!(
            "127.0.0.1:{}",
            kafka_node.get_host_port_ipv4(kafka::KAFKA_PORT)
        );

        let kafka_producer = KafkaProducer::new(bootstrap_servers.clone());

        let consumer = ClientConfig::new()
            .set("group.id", "testcontainer-rs")
            .set("bootstrap.servers", &bootstrap_servers)
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create::<StreamConsumer>()
            .expect("Failed to create Kafka StreamConsumer");
        let topic = "test-topic";
        let key = "test-key";
        let payload = Custom {
            value: "test-payload".to_string(),
        };
        let produce_result = kafka_producer
            .produce(topic, key.to_string(), payload)
            .await;
        assert!(produce_result);

        consumer
            .subscribe(&[topic])
            .expect("Failed to subscribe to the topic");

        let mut message_stream = consumer.stream();
        let borrowed_message = tokio::time::timeout(Duration::from_secs(10), message_stream.next())
            .await
            .unwrap()
            .unwrap();
        let y = borrowed_message.unwrap();
        let consumed_bytes = y.payload_view::<[u8]>().unwrap().unwrap();
        let consumed_custom: Custom = serde_json::from_slice(consumed_bytes).unwrap();
        assert_eq!(
            Custom {
                value: "test-payload".to_string()
            },
            consumed_custom
        );
    }
}
