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
