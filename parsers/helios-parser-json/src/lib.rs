use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use serde_json::Value;

pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for JsonParser {
    fn name(&self) -> &'static str {
        "json"
    }

    fn detect(&self, raw: &str) -> bool {
        serde_json::from_str::<Value>(raw).is_ok()
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let value: Value = serde_json::from_str(raw)
            .map_err(|e| Error::ParseError(format!("Invalid JSON: {}", e)))?;

        let mut event = UniversalEvent::default();
        event.raw_event = raw.to_string();

        if let Some(obj) = value.as_object() {
            if let Some(msg) = obj.get("message").and_then(|v| v.as_str()) {
                event.message = msg.to_string();
            }
            if let Some(level) = obj
                .get("level")
                .or(obj.get("severity"))
                .and_then(|v| v.as_str())
            {
                event.severity = Some(level.to_string());
            }
        }

        Ok(event)
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Generic JSON log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}
