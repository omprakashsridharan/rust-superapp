use database::get_connection;
use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let connection = get_connection("postgres://postgres:postgres@localhost/rust-superapp").await?;
    Migrator::up(&connection, None).await?;
    Ok(())
}
