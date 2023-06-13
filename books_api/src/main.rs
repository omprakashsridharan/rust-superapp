mod entity;
mod repository;

use std::sync::Arc;
use database::get_connection;
use migration::{Migrator, MigratorTrait};
use crate::repository::RepositoryBuilder;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let database_connection = get_connection("postgres://postgres:postgres@localhost/rust-superapp").await?;
    Migrator::up(&database_connection, None).await?;

    let repository = RepositoryBuilder::default().database_connection(Arc::new(database_connection.clone())).build().expect("Could not build repository");

    Ok(())
}
