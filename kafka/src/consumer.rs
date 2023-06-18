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
    topic: String,
}

impl KafkaConsumer {
    pub fn new(bootstrap_servers: String, group_id: String, topic: String) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", bootstrap_servers)
            .set("session.timeout.ms", "6000")
            .set("message.timeout.ms", "5000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .expect("Consumer creation error");
        Self { consumer, topic }
    }

    pub async fn consume<T: Debug + for<'a> Deserialize<'a>>(&self, sender: UnboundedSender<T>) {
        self.consumer
            .subscribe(&[&self.topic])
            .expect("Can't subscribe to specific topics");

        while let Ok(message) = self.consumer.recv().await {
            match message.payload_view::<[u8]>() {
                Some(Ok(bytes)) => {
                    if let Ok(deserialized_payload) = serde_json::from_slice(bytes) {
                        info!(
                                    "key: '{:?}', payload: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                                    message.key(),
                                    deserialized_payload,
                                    message.topic(),
                                    message.partition(),
                                    message.offset(),
                                    message.timestamp()
                                );
                        if let Err(e) = sender.send(deserialized_payload) {
                            error!("Error while sending via channel: {}", e);
                        } else {
                            info!("Message consumed successfully");
                        }
                    } else {
                        warn!("Error while deserializing message payload");
                    }
                }
                Some(Err(e)) => {
                    warn!("Error while deserializing message payload: {:?}", e);
                }
                None => {
                    info!("No payload");
                }
            }
            self.consumer
                .commit_message(&message, CommitMode::Async)
                .unwrap();
        }
    }
}
