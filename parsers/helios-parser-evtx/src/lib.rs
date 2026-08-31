use chrono::{DateTime, Utc};
use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use serde_json::Value;

pub struct EvtxParser;

impl EvtxParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EvtxParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for EvtxParser {
    fn name(&self) -> &'static str {
        "evtx"
    }

    fn detect(&self, raw: &str) -> bool {
        // Fast path detection: Check if it looks like EVTX JSON converted by our ingest layer
        if !raw.contains("\"Event\":") || !raw.contains("\"System\":") {
            return false;
        }
        serde_json::from_str::<Value>(raw)
            .map(|v| v.get("Event").is_some())
            .unwrap_or(false)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let value: Value = serde_json::from_str(raw)
            .map_err(|e| Error::ParseError(format!("Invalid EVTX JSON: {}", e)))?;

        let event_obj = value
            .get("Event")
            .and_then(|v| v.as_object())
            .ok_or_else(|| Error::ParseError("Missing 'Event' object".into()))?;

        let system_obj = event_obj
            .get("System")
            .and_then(|v| v.as_object())
            .ok_or_else(|| Error::ParseError("Missing 'System' object".into()))?;

        // Extract timestamp
        let timestamp = system_obj
            .get("TimeCreated")
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("#attributes"))
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("SystemTime"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .unwrap_or_else(Utc::now);

        // Extract hostname
        let hostname = system_obj
            .get("Computer")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract service / provider
        let service = system_obj
            .get("Provider")
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("#attributes"))
            .and_then(|v| v.as_object())
            .and_then(|obj| obj.get("Name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract severity level (1=CRIT, 2=ERROR, 3=WARN, 4=INFO, 5=DEBUG)
        let severity = system_obj
            .get("Level")
            .and_then(|v| v.as_u64())
            .map(|level| match level {
                1 => "CRIT".to_string(),
                2 => "ERROR".to_string(),
                3 => "WARN".to_string(),
                4 => "INFO".to_string(),
                5 => "DEBUG".to_string(),
                _ => "INFO".to_string(),
            });

        // EventData / UserData payload
        let message = event_obj
            .get("EventData")
            .or_else(|| event_obj.get("UserData"))
            .map(|v| v.to_string())
            .unwrap_or_else(|| "No EventData".to_string());

        Ok(UniversalEvent {
            timestamp,
            hostname,
            service,
            severity,
            message,
            raw_event: raw.to_string(),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Windows EVTX (XML/JSON) log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}
