mod dto;
mod entity;
mod repository;
mod service;

use crate::repository::Repository;
use database::get_connection;
use service::{book_created_producer::BookCreatedProducer, Service};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let database_connection =
        get_connection("postgres://postgres:postgres@localhost/rust-superapp").await?;
    let repository = Repository::new(database_connection.clone())
        .await
        .expect("Error creating repository");
    let book_created_producer = BookCreatedProducer::new("localhost:9092".to_owned());
    let service = Service::new(repository, book_created_producer);
    let created_book = service
        .create_and_publish_book("Title2".to_string(), "ISBN".to_string())
        .await?;
    info!("Created book {:?}", created_book);
    Ok(())
}
