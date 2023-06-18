use opentelemetry::trace::Tracer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
pub async fn setup_tracing() {
    opentelemetry::global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
    let tracer = opentelemetry_zipkin::new_pipeline()
        .with_service_name("service".to_owned())
        .with_service_address("127.0.0.1:8080".parse().unwrap())
        .with_collector_endpoint("http://localhost:9411/api/v2/spans")
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("unable to install zipkin tracer");
    tracer.in_span("tracing_init", |cx| info!("Tracing initialised"));
    let tracer = tracing_opentelemetry::layer().with_tracer(tracer.clone());

    let subscriber = tracing_subscriber::fmt::layer().json();

    let level = EnvFilter::new("debug".to_owned());

    tracing_subscriber::registry()
        .with(subscriber)
        .with(level)
        .with(tracer)
        .init();
}
