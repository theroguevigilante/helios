//! # Windows Log Parser
//!
//! Parses Windows application logs:
//!
//! ```text
//! 2024-01-15 10:23:45, INFO ComponentName Service started successfully
//! ```

use chrono::{DateTime, NaiveDateTime, Utc};
use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\s+[A-Za-z]+\s+\S+\s+")
        .expect("invalid detect regex")
});

static CAPTURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}),\s+(\w+)\s+(\S+)\s+(.*)$")
        .expect("invalid capture regex")
});

fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
}

fn normalize_severity(level: &str) -> String {
    level.to_uppercase()
}

pub struct WindowsParser;

impl WindowsParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for WindowsParser {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = CAPTURE_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match Windows log format".into()))?;

        let timestamp_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let level = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let component = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let message = caps.get(4).map(|m| m.as_str()).unwrap_or("");

        let timestamp = parse_timestamp(timestamp_str).unwrap_or_else(Utc::now);

        Ok(UniversalEvent {
            timestamp,
            hostname: None,
            service: Some(component.to_string()),
            severity: Some(normalize_severity(level)),
            message: message.to_string(),
            raw_event: raw.to_string(),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Windows log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "2024-01-15 10:23:45, INFO ComponentName Service started successfully";
    const CBS_SAMPLE: &str = "2016-09-28 04:30:30, Info                  CBS    Loaded Servicing Stack v6.1.7601.23505 with Core: C:\\Windows\\cbscore.dll";

    #[test]
    fn test_detect() {
        let parser = WindowsParser::new();
        assert!(parser.detect(SAMPLE));
        assert!(parser.detect(CBS_SAMPLE));
    }

    #[test]
    fn test_detect_rejects_other() {
        let parser = WindowsParser::new();
        assert!(!parser.detect(r#"{"message": "just json"}"#));
    }

    #[test]
    fn test_parse() {
        let parser = WindowsParser::new();
        let event = parser.parse(SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(event.service.as_deref(), Some("ComponentName"));
        assert_eq!(event.message, "Service started successfully");
        assert_eq!(event.raw_event, SAMPLE);
    }

    #[test]
    fn test_parse_cbs_multi_space() {
        let parser = WindowsParser::new();
        let event = parser.parse(CBS_SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(event.service.as_deref(), Some("CBS"));
        assert_eq!(
            event.message,
            "Loaded Servicing Stack v6.1.7601.23505 with Core: C:\\Windows\\cbscore.dll"
        );
    }
}
