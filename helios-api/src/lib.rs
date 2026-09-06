use axum::extract::DefaultBodyLimit;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use helios_core::UniversalEvent;
use helios_detector::FormatDetector;
use helios_ingest::{EvtxFileSource, LogSource};
use helios_lisp::{default_lisp_dir, load_lisp_parsers, watch_and_register};
use helios_parser::Registry;
use helios_parser_android::AndroidParser;
use helios_parser_apache::ApacheParser;
use helios_parser_cef::CefParser;
use helios_parser_evtx::EvtxParser;
use helios_parser_json::JsonParser;
use helios_parser_leef::LeefParser;
use helios_parser_nginx::NginxParser;
use helios_parser_openssh::OpenSshParser;
use helios_parser_proxifier::ProxifierParser;
use helios_parser_spark::SparkParser;
use helios_parser_syslog::SyslogParser;
use helios_parser_windows::WindowsParser;
use helios_parser_zookeeper::ZooKeeperParser;

use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    registry: Arc<RwLock<Registry>>,
    tx: broadcast::Sender<UniversalEvent>,
}

pub async fn run_server(port: u16) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut registry = Registry::new();
    registry.register(CefParser::new());
    registry.register(LeefParser::new());
    registry.register(EvtxParser::new());
    registry.register(OpenSshParser::new());
    registry.register(SyslogParser::new());
    registry.register(ApacheParser::new());
    registry.register(NginxParser::new());
    registry.register(ZooKeeperParser::new());
    registry.register(SparkParser::new());
    registry.register(WindowsParser::new());
    registry.register(AndroidParser::new());
    registry.register(ProxifierParser::new());

    registry.register(JsonParser::new());

    // Load existing Lisp parser extensions
    let lisp_dir = default_lisp_dir();
    if lisp_dir.exists() {
        for parser in load_lisp_parsers(&lisp_dir) {
            registry.register_boxed(Box::new(parser));
        }
    }

    let (tx, _rx) = broadcast::channel(10000);

    let state = AppState {
        registry: Arc::new(RwLock::new(registry)),
        tx,
    };

    // Start hot-reload watcher (keeps running in background)
    let _watcher = if lisp_dir.exists() {
        watch_and_register(&lisp_dir, state.registry.clone()).ok()
    } else {
        None
    };

    let app = Router::new()
        .route("/api/v1/stats", get(get_stats))
        .route("/api/v1/events", post(ingest_live))
        .route("/api/v1/stream", get(stream_events))
        .route("/api/v1/upload", post(upload_logs))
        .route("/api/v1/parsers", get(list_parsers))
        .layer(cors)
        .layer(DefaultBodyLimit::disable())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Starting Helios API server on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "online",
        "processed_events": 0,
        "active_parsers": state.registry.read().unwrap().parsers().len()
    }))
}

async fn ingest_live(State(state): State<AppState>, body: String) -> impl IntoResponse {
    let reg_guard = state.registry.read().unwrap();
    let detector = FormatDetector::new(&reg_guard);
    let mut parsed_count = 0;

    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(parser_name) = detector.detect(line) {
            if let Some(parser) = reg_guard.parsers().iter().find(|p| p.name() == parser_name) {
                if let Ok(event) = parser.parse(line) {
                    let _ = state.tx.send(event);
                    parsed_count += 1;
                }
            }
        }
    }
    Json(serde_json::json!({"status": "ok", "ingested": parsed_count}))
}

async fn stream_events(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| match res {
        Ok(event) => {
            let json = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(json)))
        }
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

async fn upload_logs(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UniversalEvent>>, (StatusCode, String)> {
    let mut events = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let file_name = field.file_name().unwrap_or("").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if file_name.ends_with(".evtx") {
            use std::io::Write;
            let mut temp_file = tempfile::NamedTempFile::new()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            temp_file
                .write_all(&data)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            let source = EvtxFileSource::new(temp_file.path().to_string_lossy().to_string());
            let (tx, mut rx) = tokio::sync::mpsc::channel(1000);

            tokio::spawn(async move {
                let _ = source.run(tx).await;
            });

            while let Some(line) = rx.recv().await {
                // Acquire lock per line to avoid holding across await
                let reg_guard = state.registry.read().unwrap();
                let detector = FormatDetector::new(&reg_guard);
                if let Some(parser_name) = detector.detect(&line) {
                    if let Some(parser) =
                        reg_guard.parsers().iter().find(|p| p.name() == parser_name)
                    {
                        if let Ok(event) = parser.parse(&line) {
                            events.push(event);
                        }
                    }
                }
            }
        } else {
            let body = String::from_utf8_lossy(&data);

            // We can acquire lock for the whole body since there are no awaits here
            let reg_guard = state.registry.read().unwrap();
            let detector = FormatDetector::new(&reg_guard);

            for line in body.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Some(parser_name) = detector.detect(line) {
                    if let Some(parser) =
                        reg_guard.parsers().iter().find(|p| p.name() == parser_name)
                    {
                        if let Ok(event) = parser.parse(line) {
                            events.push(event);
                        }
                    }
                }
            }
        }
    }

    Ok(Json(events))
}

async fn list_parsers(State(state): State<AppState>) -> impl IntoResponse {
    let registry = state.registry.read().unwrap();
    let parsers: Vec<serde_json::Value> = registry
        .parsers()
        .iter()
        .map(|p| {
            let meta = p.metadata();
            serde_json::json!({
                "name": p.name(),
                "type": if meta.author == "Lisp Extension" { "lisp" } else { "native" },
                "version": meta.version,
                "description": meta.description,
            })
        })
        .collect();
    Json(parsers)
}
