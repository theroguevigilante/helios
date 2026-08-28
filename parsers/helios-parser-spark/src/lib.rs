//! # Spark Log Parser
//!
//! Parses Apache Spark logs:
//!
//! ```text
//! 24/01/15 10:23:45 INFO SparkContext: Running Spark version 3.5.0
//! ```

use chrono::{DateTime, NaiveDateTime, Utc};
use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{2}/\d{2}/\d{2} \d{2}:\d{2}:\d{2} [A-Za-z]+ \S+:")
        .expect("invalid detect regex")
});

static CAPTURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{2}/\d{2}/\d{2} \d{2}:\d{2}:\d{2}) (\w+) (\S+): (.*)$")
        .expect("invalid capture regex")
});

fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(ts, "%y/%m/%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
}

fn normalize_severity(level: &str) -> String {
    level.to_uppercase()
}

pub struct SparkParser;

impl SparkParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SparkParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for SparkParser {
    fn name(&self) -> &'static str {
        "spark"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = CAPTURE_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match Spark log format".into()))?;

        let timestamp_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let level = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let logger = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let message = caps.get(4).map(|m| m.as_str()).unwrap_or("");

        let timestamp = parse_timestamp(timestamp_str).unwrap_or_else(Utc::now);

        Ok(UniversalEvent {
            timestamp,
            hostname: None,
            service: Some(logger.to_string()),
            severity: Some(normalize_severity(level)),
            message: message.to_string(),
            raw_event: raw.to_string(),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Apache Spark log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "24/01/15 10:23:45 INFO SparkContext: Running Spark version 3.5.0";

    #[test]
    fn test_detect() {
        let parser = SparkParser::new();
        assert!(parser.detect(SAMPLE));
    }

    #[test]
    fn test_detect_rejects_other() {
        let parser = SparkParser::new();
        assert!(!parser.detect(r#"{"message": "just json"}"#));
    }

    #[test]
    fn test_parse() {
        let parser = SparkParser::new();
        let event = parser.parse(SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(event.service.as_deref(), Some("SparkContext"));
        assert_eq!(event.message, "Running Spark version 3.5.0");
        assert_eq!(event.raw_event, SAMPLE);
    }
}
