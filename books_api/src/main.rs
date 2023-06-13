mod entity;
mod repository;

use database::get_connection;
use crate::repository::{Repository};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let database_connection = get_connection("postgres://postgres:postgres@localhost/rust-superapp").await?;
    let repository = Repository::new(database_connection.clone()).await.expect("Error creating repository");

    Ok(())
}
