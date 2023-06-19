use apache_avro::from_value;
use opentelemetry::{
    global,
    trace::{Span, Tracer},
    Context,
};
use rdkafka::{
    config::RDKafkaLogLevel,
    consumer::{CommitMode, Consumer, StreamConsumer},
    ClientConfig, Message,
};
use schema_registry_converter::async_impl::{
    easy_avro::EasyAvroDecoder, schema_registry::SrSettings,
};
use serde::Deserialize;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

use crate::shared::HeaderExtractor;

pub struct KafkaConsumer {
    consumer: StreamConsumer,
    avro_decoder: EasyAvroDecoder,
    topic: String,
}

impl KafkaConsumer {
    pub fn new(
        bootstrap_servers: String,
        schema_registry_url: String,
        group_id: String,
        topic: String,
    ) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", bootstrap_servers)
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .expect("Consumer creation error");
        let sr_settings = SrSettings::new(schema_registry_url);
        let avro_decoder = EasyAvroDecoder::new(sr_settings);
        Self {
            consumer,
            topic,
            avro_decoder,
        }
    }

    pub async fn consume<T: Clone + Debug + for<'a> Deserialize<'a>>(
        &self,
        sender: UnboundedSender<T>,
    ) {
        self.consumer
            .subscribe(&[&self.topic])
            .expect("Can't subscribe to specific topics");

        while let Ok(message) = self.consumer.recv().await {
            let context = if let Some(headers) = message.headers() {
                global::get_text_map_propagator(|propagator| {
                    propagator.extract(&HeaderExtractor(&headers))
                })
            } else {
                Context::current()
            };

            let mut span =
                global::tracer("consumer").start_with_context("consume_payload", &context);

            // let key = match self.avro_decoder.decode(message.key()).await {
            //     Ok(v) => v.value,
            //     Err(e) => {
            //         warn!(
            //             "encountered error, key probably was not avro encoded: {}",
            //             e
            //         );
            //         match String::from_utf8(Vec::from(message.key().unwrap())) {
            //             Ok(s) => Value::String(s),
            //             Err(_) => {
            //                 println!("It was not a String either :(");
            //                 Value::Bytes(Vec::from(message.key().unwrap()))
            //             }
            //         }
            //     }
            // };
            let value_result = match self.avro_decoder.decode(message.payload()).await {
                Ok(v) => Ok(v.value),
                Err(e) => {
                    error!("Error getting value: {}", e);
                    Err(e)
                }
            };

            if let Ok(value) = value_result {
                if let Ok(deserialized_payload) = from_value::<T>(&value) {
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
                    error!("Error while deserializing message payload");
                }
            } else {
                error!("Error while deserializing message payload");
            }
            self.consumer
                .commit_message(&message, CommitMode::Async)
                .unwrap();
            span.end();
        }
    }
}
