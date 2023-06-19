use crate::shared;
use opentelemetry::trace::{Span, TraceContextExt, Tracer};
use opentelemetry::{global, Context, Key, KeyValue, StringValue};
use rdkafka::message::{Header, OwnedHeaders, ToBytes};
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
        let mut span = global::tracer("producer").start("produce_to_kafka");
        span.set_attribute(KeyValue {
            key: Key::new("topic"),
            value: opentelemetry::Value::String(StringValue::from(self.topic.clone())),
        });
        span.set_attribute(KeyValue {
            key: Key::new("payload"),
            value: opentelemetry::Value::String(StringValue::from(
                serde_json::to_string(&payload).expect("Failed to serialize payload"),
            )),
        });
        let context = Context::current_with_span(span);
        let mut headers = OwnedHeaders::new().insert(Header {
            key: "key",
            value: Some("value"),
        });
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&context, &mut shared::HeaderInjector(&mut headers))
        });

        let record = FutureRecord::to(&self.topic)
            .payload(&serialized_data)
            .key(&key)
            .headers(headers);

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
