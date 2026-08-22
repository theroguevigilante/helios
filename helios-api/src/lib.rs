use axum::{
    routing::{get, post},
    Router,
};
use tracing::info;

pub async fn run_server(port: u16) -> Result<(), std::io::Error> {
    let api_v1 = Router::new()
        .route("/health", get(health_check))
        .route("/parsers", get(list_parsers))
        .route("/events", post(ingest_events).get(list_events))
        .route("/search", post(search_events))
        .route("/stats", get(get_statistics));

    let app = Router::new().nest("/api/v1", api_v1);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Starting Helios API server on {}", addr);
    axum::serve(listener, app).await
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_parsers() -> &'static str {
    "[]"
}

async fn ingest_events() -> &'static str {
    "Ingested"
}

async fn list_events() -> &'static str {
    "[]"
}

async fn search_events() -> &'static str {
    "[]"
}

async fn get_statistics() -> &'static str {
    "{}"
}
