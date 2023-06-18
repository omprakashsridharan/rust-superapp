mod dto;
mod entity;
mod repository;
mod service;

use crate::repository::Repository;
use database::get_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let database_connection =
        get_connection("postgres://postgres:postgres@localhost/rust-superapp").await?;
    let repository = Repository::new(database_connection.clone())
        .await
        .expect("Error creating repository");

    Ok(())
}
