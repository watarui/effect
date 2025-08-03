//! テレメトリ（ロギング、メトリクス、トレーシング）
//!
//! 全マイクロサービスで共通のテレメトリ設定

use opentelemetry::{KeyValue, trace::TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::Tracer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// テレメトリを初期化
pub fn init_telemetry(
    service_name: &str,
    otlp_endpoint: Option<&str>,
) -> Result<Tracer, Box<dyn std::error::Error>> {
    // OpenTelemetry の設定
    let resource = Resource::new(vec![KeyValue::new(
        "service.name",
        service_name.to_string(),
    )]);

    let tracer = if let Some(endpoint) = otlp_endpoint {
        use opentelemetry_sdk::runtime;
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()?;

        let provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(exporter, runtime::Tokio)
            .with_resource(resource)
            .build();
        provider.tracer(service_name.to_string())
    } else {
        // ローカル開発用のトレーサー
        let provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .with_resource(resource)
            .build();
        provider.tracer(service_name.to_string())
    };

    // Tracing subscriber の設定
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.clone());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(telemetry)
        .init();

    Ok(tracer)
}

/// メトリクスを記録
#[macro_export]
macro_rules! record_metric {
    ($name:expr, $value:expr) => {
        tracing::info!(metric.name = $name, metric.value = $value, "metric");
    };
}

/// イベントを記録
#[macro_export]
macro_rules! record_event {
    ($name:expr, $($key:tt = $value:expr),*) => {
        tracing::info!(
            event.name = $name,
            $($key = $value,)*
            "event"
        );
    };
}
