use std::time::Duration;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, Response, header},
};
use opentelemetry::{
    KeyValue, global,
    propagation::Extractor,
    trace::{TraceContextExt as _, TracerProvider as _},
};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{SdkTracerProvider, Tracer},
};
use tower_http::{
    classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier},
    trace::{DefaultOnRequest, TraceLayer},
};
use tracing::{Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{
    Layer as _, filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

use super::Config;

type TelemetryResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub type HttpTraceLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    fn(&Request<Body>) -> Span,
    DefaultOnRequest,
    fn(&Response<Body>, Duration, &Span),
    (),
    (),
    fn(ServerErrorsFailureClass, Duration, &Span),
>;

pub fn init(config: &Config) -> TelemetryResult<Option<SdkTracerProvider>> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    if otlp_enabled() {
        let provider = tracer_provider()?;
        let tracer = provider.tracer(env!("CARGO_PKG_NAME"));
        // Make the provider globally addressable so OTel-aware components
        // (reqwest-tracing, future job instrumentation)
        // resolve the same tracer instead of the no-op default.
        global::set_tracer_provider(provider.clone());
        init_subscriber(config, Some(tracer));
        Ok(Some(provider))
    } else {
        init_subscriber(config, None);
        Ok(None)
    }
}

pub fn http_trace_layer() -> HttpTraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(make_http_span as fn(&Request<Body>) -> Span)
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(on_http_response as fn(&Response<Body>, Duration, &Span))
        .on_body_chunk(())
        .on_eos(())
        .on_failure(on_http_failure as fn(ServerErrorsFailureClass, Duration, &Span))
}

fn init_subscriber(config: &Config, tracer: Option<Tracer>) {
    macro_rules! json_fmt_layer {
        () => {
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_level(true)
                .with_file(true)
                .with_line_number(true)
                // The JSON formatter writes the current span *and* the span
                // list by default, so every line inside a request repeats the
                // whole `http.server` span twice. `spans` is the one that keeps
                // the parent chain, so the current-span copy is the one to drop.
                .with_current_span(false)
                .with_filter(LevelFilter::from_level(config.log_level))
        };
    }

    match tracer {
        Some(tracer) => tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(json_fmt_layer!())
            .init(),
        None => tracing_subscriber::registry()
            .with(json_fmt_layer!())
            .init(),
    }
}

fn tracer_provider() -> TelemetryResult<SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;

    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string());

    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", service_name),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])
                .build(),
        )
        .build())
}

fn otlp_enabled() -> bool {
    std::env::var_os("OTEL_EXPORTER_OTLP_ENDPOINT").is_some()
        || std::env::var_os("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").is_some()
}

fn make_http_span(request: &Request<Body>) -> Span {
    let method = request.method();
    let matched_route = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(axum::extract::MatchedPath::as_str);

    // OTel HTTP server span name convention is "{METHOD} {http.route}" when the
    // templated route is known, otherwise just "{METHOD}". Using the raw URI
    // path would explode cardinality (one span name per id) and is what hides
    // these spans from service-map style views (groundcover, Tempo, etc.).
    let route_for_name = matched_route.unwrap_or("");
    let span_name = if route_for_name.is_empty() {
        method.to_string()
    } else {
        format!("{method} {route_for_name}")
    };

    // A push token in the path is a secret: such a request is logged with
    // its route template instead of the path.
    let path = match matched_route {
        Some(route) if route.contains("{push_token}") => route,
        _ => request.uri().path(),
    };
    let scheme = request.uri().scheme_str().unwrap_or("http").to_string();
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let user_agent = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let span = tracing::info_span!(
        "http.server",
        otel.name = %span_name,
        otel.kind = "server",
        otel.status_code = tracing::field::Empty,
        // Current OTel HTTP semconv.
        http.request.method = %method,
        http.route = matched_route.unwrap_or_default(),
        http.response.status_code = tracing::field::Empty,
        url.path = %path,
        url.query = request.uri().query().unwrap_or_default(),
        url.scheme = %scheme,
        server.address = %host,
        user_agent.original = %user_agent,
        // Legacy aliases, kept because several backends (incl. older
        // groundcover views) still key off these names to recognize HTTP
        // server spans and to render them on the service map.
        http.method = %method,
        http.target = %path,
        http.scheme = %scheme,
        http.host = %host,
        http.status_code = tracing::field::Empty,
    );

    // Only stitch into a remote trace when the caller actually sent W3C
    // context. Calling `set_parent` with an empty context detaches this span
    // from the runtime's local trace context, which breaks parent/child
    // stitching for any spans created inside the handler (and made every
    // server span look orphaned in groundcover).
    let parent_context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });
    if parent_context.span().span_context().is_valid() {
        let _ = span.set_parent(parent_context);
    }

    span
}

fn on_http_response(response: &Response<Body>, latency: Duration, span: &Span) {
    let status = response.status();
    span.record("http.response.status_code", status.as_u16());
    span.record("http.status_code", status.as_u16());
    if status.is_server_error() {
        span.record("otel.status_code", "ERROR");
    }
    let _enter = span.enter();
    tracing::info!(
        latency_ms = latency.as_millis() as u64,
        status = status.as_u16(),
        "finished processing request"
    );
}

fn on_http_failure(failure: ServerErrorsFailureClass, latency: Duration, span: &Span) {
    span.record("otel.status_code", "ERROR");
    let _enter = span.enter();
    tracing::error!(
        latency_ms = latency.as_millis() as u64,
        failure = %failure,
        "request failed"
    );
}

struct HeaderExtractor<'a>(&'a HeaderMap);

impl Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|key| key.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct CapturedLines(Arc<Mutex<Vec<u8>>>);

    impl CapturedLines {
        fn json(&self) -> Vec<serde_json::Value> {
            let raw = self.0.lock().expect("captured lines");
            String::from_utf8_lossy(&raw)
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| serde_json::from_str(line).expect("each line is json"))
                .collect()
        }
    }

    impl std::io::Write for CapturedLines {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("captured lines")
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for CapturedLines {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    /// Emits one event inside a span, through the same formatter configuration
    /// `init_subscriber` builds.
    fn log_inside_a_span(current_span: bool) -> Vec<serde_json::Value> {
        let captured = CapturedLines::default();

        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_target(false)
            .with_current_span(current_span)
            .with_writer(captured.clone())
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("http.server", http.route = "/v1/farms");
            let _enter = span.enter();
            tracing::info!(status = 200, "finished processing request");
        });

        captured.json()
    }

    #[test]
    fn a_log_line_does_not_repeat_its_span_twice() {
        let lines = log_inside_a_span(false);

        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].get("span").is_none(),
            "the current-span copy duplicates what `spans` already carries"
        );
    }

    #[test]
    fn the_span_chain_is_still_reachable_from_a_log_line() {
        let lines = log_inside_a_span(false);

        let spans = lines[0]["spans"]
            .as_array()
            .expect("the span list survives so the route stays attached");

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0]["http.route"], "/v1/farms");
    }

    #[test]
    fn the_default_really_would_have_duplicated_it() {
        let lines = log_inside_a_span(true);

        assert_eq!(
            lines[0]["span"]["http.route"], lines[0]["spans"][0]["http.route"],
            "this is the duplication with_current_span(false) removes"
        );
    }
}
