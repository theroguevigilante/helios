//! # Apache Access Log Parser
//!
//! Parses Apache HTTP Server access logs in Combined Log Format:
//!
//! ```text
//! %h %l %u %t "%r" %>s %b "%{Referer}i" "%{User-Agent}i"
//! ```
//!
//! Also supports the Common Log Format (without referer and user-agent).

use chrono::{DateTime, FixedOffset, Utc};
use helios_core::{Error, NetworkInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

// Combined Log Format regex:
// 192.168.1.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /page.html HTTP/1.1" 200 2326 "http://ref.com" "Mozilla/5.0"
//
// Also matches Common Log Format (no referer/user-agent fields).
static COMBINED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(\S+) (\S+) (\S+) \[([^\]]+)\] "([^"]*)" (\d{3}) (\S+)(?: "(.*)" "(.*)")?$"#)
        .expect("invalid apache log regex")
});

// Detection regex — looser version just to check if a line looks like an access log
static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\S+ \S+ \S+ \[\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2} [+\-]\d{4}\] ""#)
        .expect("invalid detect regex")
});

fn parse_clf_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_str(ts, "%d/%b/%Y:%H:%M:%S %z")
        .ok()
        .map(|dt: DateTime<FixedOffset>| dt.with_timezone(&Utc))
}

fn status_to_severity(status: u16) -> &'static str {
    match status {
        200..=299 => "INFO",
        300..=399 => "INFO",
        400..=499 => "WARN",
        500..=599 => "ERROR",
        _ => "INFO",
    }
}

pub struct ApacheParser;

impl Default for ApacheParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ApacheParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ApacheParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for ApacheParser {
    fn name(&self) -> &'static str {
        "apache"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = COMBINED_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match Apache access log format".into()))?;

        let remote_host = caps.get(1).map(|m| m.as_str()).unwrap_or("-");
        let _ident = caps.get(2).map(|m| m.as_str()).unwrap_or("-");
        let remote_user = caps.get(3).map(|m| m.as_str()).unwrap_or("-");
        let timestamp_str = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let request = caps.get(5).map(|m| m.as_str()).unwrap_or("");
        let status_str = caps.get(6).map(|m| m.as_str()).unwrap_or("0");
        let bytes_str = caps.get(7).map(|m| m.as_str()).unwrap_or("-");
        let referer = caps.get(8).map(|m| m.as_str());
        let user_agent = caps.get(9).map(|m| m.as_str());

        let timestamp = parse_clf_timestamp(timestamp_str).unwrap_or_else(Utc::now);
        let status: u16 = status_str.parse().unwrap_or(0);

        let mut attributes = std::collections::HashMap::new();
        attributes.insert(
            "http.request".to_string(),
            serde_json::Value::String(request.to_string()),
        );
        attributes.insert(
            "http.status_code".to_string(),
            serde_json::Value::Number(status.into()),
        );
        if bytes_str != "-" {
            if let Ok(bytes) = bytes_str.parse::<u64>() {
                attributes.insert(
                    "http.response_bytes".to_string(),
                    serde_json::Value::Number(bytes.into()),
                );
            }
        }
        if remote_user != "-" {
            attributes.insert(
                "user.name".to_string(),
                serde_json::Value::String(remote_user.to_string()),
            );
        }
        if let Some(r) = referer {
            if r != "-" {
                attributes.insert(
                    "http.referer".to_string(),
                    serde_json::Value::String(r.to_string()),
                );
            }
        }
        if let Some(ua) = user_agent {
            attributes.insert(
                "user_agent.original".to_string(),
                serde_json::Value::String(ua.to_string()),
            );
        }

        Ok(UniversalEvent {
            timestamp,
            hostname: None,
            service: Some("apache".to_string()),
            severity: Some(status_to_severity(status).to_string()),
            message: format!("{} → {}", request, status),
            raw_event: raw.to_string(),
            network: Some(NetworkInfo {
                source_ip: Some(remote_host.to_string()),
                source_port: None,
                destination_ip: None,
                destination_port: None,
                protocol: Some("HTTP".to_string()),
            }),
            attributes,
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Apache HTTP Server Combined/Common Log Format Parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMBINED: &str = r##"192.168.1.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326 "http://www.example.com/start.html" "Mozilla/4.08 [en] (Win98; I)""##;

    const COMMON: &str =
        r#"127.0.0.1 - - [28/Aug/2026:10:27:32 +0530] "POST /api/events HTTP/1.1" 201 45"#;

    const ERROR_STATUS: &str = r#"10.0.0.5 - admin [28/Aug/2026:10:30:00 +0000] "GET /admin/config HTTP/1.1" 403 1234 "-" "curl/7.68.0""#;

    const SERVER_ERROR: &str = r#"172.16.0.1 - - [28/Aug/2026:10:35:00 +0000] "GET /crash HTTP/1.1" 500 0 "-" "python-requests/2.28.0""#;

    #[test]
    fn test_detect_combined() {
        let parser = ApacheParser::new();
        assert!(parser.detect(COMBINED));
    }

    #[test]
    fn test_detect_common() {
        let parser = ApacheParser::new();
        assert!(parser.detect(COMMON));
    }

    #[test]
    fn test_detect_rejects_syslog() {
        let parser = ApacheParser::new();
        assert!(!parser.detect("<34>Oct 11 22:14:15 mymachine su: test"));
    }

    #[test]
    fn test_detect_rejects_json() {
        let parser = ApacheParser::new();
        assert!(!parser.detect(r#"{"message": "hello"}"#));
    }

    #[test]
    fn test_parse_combined() {
        let parser = ApacheParser::new();
        let event = parser.parse(COMBINED).unwrap();

        assert_eq!(event.severity.unwrap(), "INFO");
        assert_eq!(event.message, "GET /apache_pb.gif HTTP/1.0 → 200");

        let net = event.network.unwrap();
        assert_eq!(net.source_ip.unwrap(), "192.168.1.1");

        assert_eq!(
            event.attributes.get("user.name").unwrap(),
            &serde_json::Value::String("frank".into())
        );
        assert_eq!(
            event.attributes.get("http.status_code").unwrap(),
            &serde_json::json!(200)
        );
        assert_eq!(
            event.attributes.get("http.response_bytes").unwrap(),
            &serde_json::json!(2326)
        );
        assert!(event.attributes.contains_key("http.referer"));
        assert!(event.attributes.contains_key("user_agent.original"));
    }

    #[test]
    fn test_parse_common_format() {
        let parser = ApacheParser::new();
        let event = parser.parse(COMMON).unwrap();

        assert_eq!(event.severity.unwrap(), "INFO");
        assert_eq!(event.message, "POST /api/events HTTP/1.1 → 201");

        let net = event.network.unwrap();
        assert_eq!(net.source_ip.unwrap(), "127.0.0.1");
        assert!(!event.attributes.contains_key("user.name"));
    }

    #[test]
    fn test_parse_403_is_warn() {
        let parser = ApacheParser::new();
        let event = parser.parse(ERROR_STATUS).unwrap();
        assert_eq!(event.severity.unwrap(), "WARN");
    }

    #[test]
    fn test_parse_500_is_error() {
        let parser = ApacheParser::new();
        let event = parser.parse(SERVER_ERROR).unwrap();
        assert_eq!(event.severity.unwrap(), "ERROR");
    }
}
