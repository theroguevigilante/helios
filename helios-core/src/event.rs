use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal Event Model inspired by Elastic Common Schema (ECS) and OpenTelemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// Hostname or IP where the event originated
    pub hostname: Option<String>,
    /// Service or application that generated the event
    pub service: Option<String>,
    /// Severity level of the event (e.g., INFO, ERROR, DEBUG)
    pub severity: Option<String>,
    /// The parsed main message of the log
    pub message: String,
    /// The original raw event line
    pub raw_event: String,
    /// Contextual metadata enriched by the platform
    pub metadata: HashMap<String, serde_json::Value>,
    /// Process information (PID, name, etc.)
    pub process: Option<ProcessInfo>,
    /// Network information (source/dest IPs, ports, etc.)
    pub network: Option<NetworkInfo>,
    /// Trace identifiers for distributed tracing correlation
    pub trace: Option<TraceInfo>,
    /// Arbitrary additional attributes extracted from the log
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: Option<u32>,
    pub name: Option<String>,
    pub thread_id: Option<u32>,
    pub thread_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub source_ip: Option<String>,
    pub source_port: Option<u16>,
    pub destination_ip: Option<String>,
    pub destination_port: Option<u16>,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceInfo {
    pub trace_id: String,
    pub span_id: Option<String>,
}

impl Default for UniversalEvent {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            hostname: None,
            service: None,
            severity: None,
            message: String::new(),
            raw_event: String::new(),
            metadata: HashMap::new(),
            process: None,
            network: None,
            trace: None,
            attributes: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_event_default() {
        let event = UniversalEvent::default();
        assert!(event.message.is_empty());
        assert!(event.raw_event.is_empty());
        assert!(event.metadata.is_empty());
        assert!(event.attributes.is_empty());
    }
}
