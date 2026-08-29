use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use helios_core::UniversalEvent;
use helios_detector::FormatDetector;
use helios_enrichment::EnrichmentPipeline;
use helios_parser::{ParserMetadata, Registry};
use helios_parser_android::AndroidParser;
use helios_parser_apache::ApacheParser;
use helios_parser_cef::CefParser;
use helios_parser_json::JsonParser;
use helios_parser_nginx::NginxParser;
use helios_parser_openssh::OpenSshParser;
use helios_parser_proxifier::ProxifierParser;
use helios_parser_spark::SparkParser;
use helios_parser_syslog::SyslogParser;
use helios_parser_windows::WindowsParser;
use helios_parser_zookeeper::ZooKeeperParser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

pub struct AppState {
    pub registry: Registry,
    pub enrichment: EnrichmentPipeline,
}

#[derive(Debug, Deserialize)]
pub struct DetectRequest {
    pub log: String,
}

#[derive(Debug, Serialize)]
pub struct DetectResponse {
    pub detected: bool,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    pub log: String,
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseResponse {
    pub format: String,
    pub event: UniversalEvent,
}

#[derive(Debug, Serialize)]
pub struct ParserInfo {
    pub name: String,
    pub metadata: ParserMetadata,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn create_router() -> Router {
    let mut registry = Registry::new();
    registry.register(JsonParser::new());
    registry.register(CefParser::new());
    registry.register(OpenSshParser::new());
    registry.register(SyslogParser::new());
    registry.register(ApacheParser::new());
    registry.register(NginxParser::new());
    registry.register(ZooKeeperParser::new());
    registry.register(SparkParser::new());
    registry.register(WindowsParser::new());
    registry.register(AndroidParser::new());
    registry.register(ProxifierParser::new());

    let state = Arc::new(AppState {
        registry,
        enrichment: EnrichmentPipeline::new(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .route("/parsers", get(list_parsers))
        .route("/detect", post(detect_format))
        .route("/parse", post(parse_log))
        .route("/normalize", post(normalize_log))
        .route("/events", post(ingest_events).get(list_events))
        .route("/search", post(search_events))
        .route("/stats", get(get_statistics))
        .with_state(state.clone());

    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_v1)
        .layer(cors)
}

pub async fn run_server(port: u16) -> Result<(), std::io::Error> {
    let app = create_router();
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Starting Helios API server on {}", addr);
    axum::serve(listener, app).await
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "helios-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn list_parsers(State(state): State<Arc<AppState>>) -> Json<Vec<ParserInfo>> {
    let parsers = state
        .registry
        .parsers()
        .iter()
        .map(|p| ParserInfo {
            name: p.name().to_string(),
            metadata: p.metadata(),
        })
        .collect();

    Json(parsers)
}

async fn detect_format(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DetectRequest>,
) -> Json<DetectResponse> {
    let detector = FormatDetector::new(&state.registry);
    let format = detector.detect(&payload.log).map(|s| s.to_string());
    let detected = format.is_some();

    Json(DetectResponse { detected, format })
}

async fn parse_log(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ParseRequest>,
) -> Result<Json<ParseResponse>, (StatusCode, Json<ErrorResponse>)> {
    let detector = FormatDetector::new(&state.registry);

    // Determine parser name: explicit or detected
    let parser_name = match payload.format.as_deref() {
        Some(name) => name.to_string(),
        None => detector
            .detect(&payload.log)
            .map(|s| s.to_string())
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Could not detect log format".to_string(),
                    }),
                )
            })?,
    };

    let parser = state
        .registry
        .parsers()
        .iter()
        .find(|p| p.name() == parser_name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Parser '{}' not found", parser_name),
                }),
            )
        })?;

    match parser.parse(&payload.log) {
        Ok(event) => Ok(Json(ParseResponse {
            format: parser_name,
            event,
        })),
        Err(e) => {
            warn!("Failed to parse log: {}", e);
            Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse {
                    error: format!("Parsing error: {}", e),
                }),
            ))
        }
    }
}

async fn normalize_log(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ParseRequest>,
) -> Result<Json<ParseResponse>, (StatusCode, Json<ErrorResponse>)> {
    let detector = FormatDetector::new(&state.registry);

    let parser_name = detector
        .detect(&payload.log)
        .map(|s| s.to_string())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Format not recognized — unable to normalize".to_string(),
                }),
            )
        })?;

    let parser = state
        .registry
        .parsers()
        .iter()
        .find(|p| p.name() == parser_name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Parser '{}' not found", parser_name),
                }),
            )
        })?;

    let mut event = parser.parse(&payload.log).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse {
                error: format!("Normalization error: {}", e),
            }),
        )
    })?;

    state.enrichment.enrich(&mut event);

    Ok(Json(ParseResponse {
        format: parser_name,
        event,
    }))
}

async fn ingest_events() -> impl IntoResponse {
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "status": "accepted" })),
    )
}

async fn list_events() -> Json<Vec<UniversalEvent>> {
    Json(Vec::new())
}

async fn search_events() -> Json<Vec<UniversalEvent>> {
    Json(Vec::new())
}

async fn get_statistics() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "online",
        "processed_events": 0,
        "active_parsers": 11
    }))
}
