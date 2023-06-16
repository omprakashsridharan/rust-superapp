use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{CommitMode, Consumer, StreamConsumer},
    ClientConfig, Message,
};
use serde::Deserialize;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info, warn};

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(bootstrap_servers: String, group_id: String) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .set("enable.auto.commit", "true")
            .set("enable.partition.eof", "false")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .expect("Consumer creation error");
        Self { consumer }
    }

    pub async fn consume<T: Debug + for<'a> Deserialize<'a>>(
        &self,
        topics: &[&str],
        sender: UnboundedSender<T>,
    ) {
        self.consumer
            .subscribe(&topics.to_vec())
            .expect("Can't subscribe to specific topics");
        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    match message.payload_view::<[u8]>() {
                        Some(Ok(bytes)) => {
                            let deserialised_payload: T = serde_json::from_slice(bytes).unwrap();
                            info!("key: '{:?}', payload: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      message.key(), deserialised_payload, message.topic(), message.partition(), message.offset(), message.timestamp());
                            match sender.send(deserialised_payload) {
                                Ok(()) => todo!(),
                                Err(e) => error!("error while sending via channel {}", e),
                            }
                        }
                        Some(Err(e)) => {
                            warn!("Error while deserializing message payload: {:?}", e);
                        }
                        None => {}
                    };
                    self.consumer
                        .commit_message(&message, CommitMode::Async)
                        .unwrap();
                }
                Err(error) => warn!("Kafka error: {}", error),
            }
        }
    }
}
