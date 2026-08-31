use async_trait::async_trait;
use helios_core::Result;
use tokio::sync::mpsc;

/// Represents a source of log data (e.g., file reader, TCP socket, Kafka consumer)
#[async_trait]
pub trait LogSource: Send + Sync {
    /// Start reading logs and sending the raw string lines to the provided channel
    async fn run(&self, tx: mpsc::Sender<String>) -> Result<()>;
}

/// Stub for a file-based log source
pub struct FileSource {
    pub path: String,
}

impl FileSource {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl LogSource for FileSource {
    async fn run(&self, _tx: mpsc::Sender<String>) -> Result<()> {
        Ok(())
    }
}

pub struct EvtxFileSource {
    pub path: String,
}

impl EvtxFileSource {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

#[async_trait]
impl LogSource for EvtxFileSource {
    async fn run(&self, tx: mpsc::Sender<String>) -> Result<()> {
        let path = self.path.clone();

        // evtx parsing is CPU intensive and blocking, so we run it in spawn_blocking
        let _ = tokio::task::spawn_blocking(move || {
            let mut parser = match evtx::EvtxParser::from_path(&path) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("Failed to open EVTX file {}: {}", path, e);
                    return;
                }
            };

            for record in parser.records_json_value() {
                match record {
                    Ok(r) => {
                        let json_str = r.data.to_string();
                        if tx.blocking_send(json_str).is_err() {
                            break; // receiver dropped
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse EVTX record: {}", e);
                    }
                }
            }
        })
        .await;

        Ok(())
    }
}
