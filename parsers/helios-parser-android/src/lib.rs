//! # Android Logcat Parser
//!
//! Parses Android logcat logs in threadtime format:
//!
//! ```text
//! 01-15 10:23:45.123 1234 5678 I ActivityManager: Starting activity
//! ```

use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use helios_core::{Error, ProcessInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}\s+\d+\s+\d+\s+[VDIWEFvdiwef]\s+\S+:")
        .expect("invalid detect regex")
});

static CAPTURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3})\s+(\d+)\s+(\d+)\s+([A-Za-z]+)\s+([^:]+):\s*(.*)$",
    )
    .expect("invalid capture regex")
});

fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    let year = Utc::now().year();
    let full_ts = format!("{}-{}", year, ts);
    NaiveDateTime::parse_from_str(&full_ts, "%Y-%m-%d %H:%M:%S%.3f")
        .ok()
        .map(|dt| dt.and_utc())
}

fn normalize_severity(level: &str) -> String {
    match level.to_uppercase().as_str() {
        "V" | "VERBOSE" => "VERBOSE".to_string(),
        "D" | "DEBUG" => "DEBUG".to_string(),
        "I" | "INFO" => "INFO".to_string(),
        "W" | "WARN" | "WARNING" => "WARN".to_string(),
        "E" | "ERROR" => "ERROR".to_string(),
        "F" | "FATAL" => "FATAL".to_string(),
        other => other.to_string(),
    }
}

pub struct AndroidParser;

impl AndroidParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AndroidParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for AndroidParser {
    fn name(&self) -> &'static str {
        "android"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = CAPTURE_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match Android log format".into()))?;

        let timestamp_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let pid_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let tid_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let level = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let tag = caps.get(5).map(|m| m.as_str()).unwrap_or("");
        let message = caps.get(6).map(|m| m.as_str()).unwrap_or("");

        let timestamp = parse_timestamp(timestamp_str).unwrap_or_else(Utc::now);
        let pid = pid_str.parse::<u32>().ok();
        let thread_id = tid_str.parse::<u32>().ok();

        Ok(UniversalEvent {
            timestamp,
            hostname: None,
            service: Some(tag.to_string()),
            severity: Some(normalize_severity(level)),
            message: message.to_string(),
            raw_event: raw.to_string(),
            process: Some(ProcessInfo {
                pid,
                thread_id,
                name: None,
                thread_name: None,
            }),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Android logcat (threadtime format) parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "01-15 10:23:45.123 1234 5678 I ActivityManager: Starting activity";

    #[test]
    fn test_detect() {
        let parser = AndroidParser::new();
        assert!(parser.detect(SAMPLE));
    }

    #[test]
    fn test_detect_rejects_other() {
        let parser = AndroidParser::new();
        assert!(!parser.detect(r#"{"message": "just json"}"#));
    }

    #[test]
    fn test_parse() {
        let parser = AndroidParser::new();
        let event = parser.parse(SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(event.service.as_deref(), Some("ActivityManager"));
        assert_eq!(event.message, "Starting activity");
        assert_eq!(event.raw_event, SAMPLE);

        let proc = event.process.unwrap();
        assert_eq!(proc.pid, Some(1234));
        assert_eq!(proc.thread_id, Some(5678));
        assert_eq!(proc.name, None);
        assert_eq!(proc.thread_name, None);
    }
}
