use common::events::{constants::Topics, dto::CreatedBook};
use kafka::consumer::KafkaConsumer;
use tokio::sync::mpsc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    opentelemetry::global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
    let tracer = opentelemetry_zipkin::new_pipeline()
        .with_service_name("books_analytics".to_owned())
        .with_service_address("127.0.0.1:8080".parse().unwrap())
        .with_collector_endpoint("http://localhost:9411/api/v2/spans")
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("unable to install zipkin tracer");
    let tracer = tracing_opentelemetry::layer().with_tracer(tracer.clone());

    let subscriber = tracing_subscriber::fmt::layer().json();

    let level = EnvFilter::new("debug".to_owned());

    tracing_subscriber::registry()
        .with(subscriber)
        .with(level)
        .with(tracer)
        .init();
    let kakfa_consumer = KafkaConsumer::new(
        "localhost:9092".to_string(),
        "http://localhost:8081".to_string(),
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
    opentelemetry::global::shutdown_tracer_provider();
    Ok(())
}
