use async_trait::async_trait;
use helios_core::{Result, UniversalEvent};

/// Abstraction layer for different storage backends (SQLite, ClickHouse, etc.)
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Initialize the storage backend (e.g., run migrations)
    async fn init(&self) -> Result<()>;

    /// Store a single event
    async fn store(&self, event: UniversalEvent) -> Result<()>;

    /// Store multiple events in a batch for higher throughput
    async fn store_batch(&self, events: Vec<UniversalEvent>) -> Result<()>;
}

/// A stub for SQLite storage to be implemented later
pub struct SqliteStorage {}

impl Default for SqliteStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteStorage {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StorageBackend for SqliteStorage {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn store(&self, _event: UniversalEvent) -> Result<()> {
        Ok(())
    }

    async fn store_batch(&self, _events: Vec<UniversalEvent>) -> Result<()> {
        Ok(())
    }
}
