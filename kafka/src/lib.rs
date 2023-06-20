pub mod consumer;
pub mod producer;
pub mod shared;

#[cfg(test)]
mod tests {

    use apache_avro::AvroSchema;
    use serde::{Deserialize, Serialize};
    use tokio::sync::mpsc;

    use crate::{
        consumer::KafkaConsumer, producer::KafkaProducer,
    };

    #[tokio::test]
    async fn test_produce() {
        let topic = "string-topic";
        let key = "test-key";
        let payload = "test-payload";

        let kafka_producer = KafkaProducer::new(
            "localhost:9092".to_string(),
            "http://localhost:8081".to_string(),
            topic.to_string(),
        );
        let kakfa_consumer = KafkaConsumer::new(
            "localhost:9092".to_string(),
            "http://localhost:8081".to_string(),
            "string-consumer".to_string(),
            topic.to_string(),
        );

        let (sender, mut receiver) = mpsc::unbounded_channel::<String>();
        let produce_result = kafka_producer
            .produce(key.to_string(), payload.to_string())
            .await;
        assert!(produce_result);
        let handle = tokio::spawn(async move {
            kakfa_consumer.consume(sender.clone()).await;
        });

        while let Some(message) = receiver.recv().await {
            assert_eq!(payload.to_string(), message);
            break;
        }
        handle.abort();
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, AvroSchema)]
    struct Custom {
        value: String,
    }

    #[tokio::test]
    async fn test_custom_struct_produce() {
        let topic = "custom-struct-topic";
        let key = "test-key";
        let payload = Custom {
            value: "test-payload".to_string(),
        };
        let kafka_producer = KafkaProducer::new(
            "localhost:9092".to_string(),
            "http://localhost:8081".to_string(),
            topic.to_string(),
        );
        let kakfa_consumer = KafkaConsumer::new(
            "localhost:9092".to_string(),
            "http://localhost:8081".to_string(),
            "custom-consumer".to_string(),
            topic.to_string(),
        );

        let produce_result = kafka_producer
            .produce(key.to_string(), payload.clone())
            .await;
        assert!(produce_result);
        let (sender, mut receiver) = mpsc::unbounded_channel::<Custom>();
        let handle = tokio::spawn(async move {
            kakfa_consumer.consume(sender.clone()).await;
        });

        while let Some(message) = receiver.recv().await {
            assert_eq!(payload, message);
            break;
        }
        handle.abort()
    }
}
