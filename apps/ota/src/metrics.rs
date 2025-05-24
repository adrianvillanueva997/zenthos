use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use tracing_subscriber::{EnvFilter, prelude::*};

use opentelemetry_stdout::LogExporter;
use tracing_subscriber::layer::Layer;

pub fn create_opentelemetry_layer() {
    let exporter = LogExporter::default();
    let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
        .with_resource(Resource::builder().with_service_name("ota-server").build())
        .with_simple_exporter(exporter)
        .build();
    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(filter_otel);
    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();
}
