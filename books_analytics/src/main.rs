use common::events::{constants::Topics, dto::CreatedBook};
use kafka::consumer::KafkaConsumer;
use tokio::sync::mpsc;
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
    let kakfa_consumer = KafkaConsumer::new(
        "localhost:9092".to_string(),
        "books-created-consumer".to_string(),
        Topics::BookCreated.to_string(),
    );

    let (sender, mut receiver) = mpsc::unbounded_channel::<CreatedBook>();
    tokio::spawn(async move {
        info!("Strarting book created consumer");
        kakfa_consumer.consume(sender.clone()).await;
    });

    while let Some(message) = receiver.recv().await {
        info!("Consumed messaged {:?}", message)
    }
    Ok(())
}
