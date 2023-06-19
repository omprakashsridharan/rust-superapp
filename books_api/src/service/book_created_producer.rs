use common::events::{
    constants::Topics,
    dto::{CreatedBookBuilder, CreatedBookBuilderError},
};
use kafka::producer::KafkaProducer;
use thiserror::Error;

#[derive(Clone)]
pub struct BookCreatedProducer {
    producer: KafkaProducer,
}

#[derive(Error, Debug)]
pub enum BookCreatedProducerError {
    #[error("CreatedBookBuilderError error")]
    CreatedBookBuilderError(#[from] CreatedBookBuilderError),
}

impl BookCreatedProducer {
    pub fn new(bootstrap_servers: String, schema_registry_url: String) -> Self {
        Self {
            producer: KafkaProducer::new(
                bootstrap_servers,
                schema_registry_url,
                Topics::BookCreated.to_string(),
            ),
        }
    }

    pub async fn publish_created_book(
        &self,
        id: i32,
        title: String,
        isbn: String,
    ) -> Result<bool, BookCreatedProducerError> {
        let created_book = CreatedBookBuilder::default()
            .id(id)
            .title(title)
            .isbn(isbn)
            .build()
            .map_err(|e| BookCreatedProducerError::CreatedBookBuilderError(e))?;
        Ok(self.producer.produce(id.to_string(), created_book).await)
    }
}
