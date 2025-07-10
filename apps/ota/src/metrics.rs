use opentelemetry::global;
use opentelemetry::trace::Tracer;
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::WithTonicConfig;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, prelude::*};

use opentelemetry_otlp::WithExportConfig;
use opentelemetry_stdout::LogExporter;
use tracing_subscriber::layer::Layer;

pub fn create_opentelemetry_layer() {
    let log_exporter = LogExporter::default();
    let logger_provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_service_name("ota-server").build())
        .with_simple_exporter(log_exporter)
        .build();

    let filter_otel_logs = EnvFilter::new("info") // Filter for what logs OpenTelemetry should process
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    let otel_log_layer: tracing_subscriber::filter::Filtered<
        layer::OpenTelemetryTracingBridge<SdkLoggerProvider, opentelemetry_sdk::logs::SdkLogger>,
        EnvFilter,
        tracing_subscriber::Registry, // This is the crucial change
    > = layer::OpenTelemetryTracingBridge::new(&logger_provider).with_filter(filter_otel_logs);

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_compression(opentelemetry_otlp::Compression::Zstd)
        .with_endpoint("http://localhost:4317")
        .build()
        .unwrap();

    let tracer_provider = SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .with_resource(Resource::builder().with_service_name("ota-server").build())
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer() // No::<tracing_subscriber::Registry> here
        .with_thread_names(true)
        .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(Box::new(otel_log_layer))
        .with(Box::new(fmt_layer))
        .init();
    let tracer = global::tracer("my_tracer");
    tracer.in_span("doing_work", |_cx| {
        println!("test: doing some work");
        tracing::info!("This is an info log from tracing inside the span!");
    });
}
