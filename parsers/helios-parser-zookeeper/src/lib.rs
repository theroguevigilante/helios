//! # ZooKeeper Log Parser
//!
//! Parses Apache ZooKeeper logs:
//!
//! ```text
//! 2024-01-15 10:23:45,123 - INFO [main:ZooKeeperServer@123] - Server started
//! ```

use chrono::{DateTime, NaiveDateTime, Utc};
use helios_core::{Error, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3}\s+-\s+").expect("invalid detect regex")
});

static CAPTURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3})\s+-\s+(\w+)\s+\[(.*)\]\s+-\s+(.*)$")
        .expect("invalid capture regex")
});

fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S,%3f")
        .ok()
        .map(|dt| dt.and_utc())
}

fn normalize_severity(level: &str) -> String {
    level.to_uppercase()
}

pub struct ZooKeeperParser;

impl ZooKeeperParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZooKeeperParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for ZooKeeperParser {
    fn name(&self) -> &'static str {
        "zookeeper"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = CAPTURE_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match ZooKeeper log format".into()))?;

        let timestamp_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let level = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let context = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let message = caps.get(4).map(|m| m.as_str()).unwrap_or("");

        let timestamp = parse_timestamp(timestamp_str).unwrap_or_else(Utc::now);

        Ok(UniversalEvent {
            timestamp,
            hostname: None,
            service: Some(context.to_string()),
            severity: Some(normalize_severity(level)),
            message: message.to_string(),
            raw_event: raw.to_string(),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Apache ZooKeeper log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str =
        "2024-01-15 10:23:45,123 - INFO [main:ZooKeeperServer@123] - Server started";
    const NESTED_SAMPLE: &str =
        "2015-07-29 17:41:44,747 - INFO  [QuorumPeer[myid=1]/0:0:0:0:0:0:0:0:2181:FastLeaderElection@774] - Notification time out: 3200";

    #[test]
    fn test_detect() {
        let parser = ZooKeeperParser::new();
        assert!(parser.detect(SAMPLE));
        assert!(parser.detect(NESTED_SAMPLE));
    }

    #[test]
    fn test_detect_rejects_other() {
        let parser = ZooKeeperParser::new();
        assert!(!parser.detect(r#"{"message": "just json"}"#));
    }

    #[test]
    fn test_parse() {
        let parser = ZooKeeperParser::new();
        let event = parser.parse(SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(event.service.as_deref(), Some("main:ZooKeeperServer@123"));
        assert_eq!(event.message, "Server started");
        assert_eq!(event.raw_event, SAMPLE);
    }

    #[test]
    fn test_parse_nested_brackets() {
        let parser = ZooKeeperParser::new();
        let event = parser.parse(NESTED_SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(
            event.service.as_deref(),
            Some("QuorumPeer[myid=1]/0:0:0:0:0:0:0:0:2181:FastLeaderElection@774")
        );
        assert_eq!(event.message, "Notification time out: 3200");
    }
}
