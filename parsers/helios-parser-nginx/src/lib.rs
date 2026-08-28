//! # Nginx Access Log Parser
//!
//! Parses Nginx access logs in the default combined format:
//!
//! ```text
//! $remote_addr - $remote_user [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent"
//! ```
//!
//! Also supports an extended format with upstream response time:
//!
//! ```text
//! ... "$http_user_agent" rt=$request_time uct=$upstream_connect_time uht=$upstream_header_time urt=$upstream_response_time
//! ```

use chrono::{DateTime, FixedOffset, Utc};
use helios_core::{Error, NetworkInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

// Standard nginx combined format + optional trailing key=value fields
static NGINX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(\S+) - (\S+) \[([^\]]+)\] "([^"]*)" (\d{3}) (\S+) "(.*)" "(.*)"(.*)$"#)
        .expect("invalid nginx log regex")
});

// Detection: combined format with exactly a dash for ident field (nginx always uses `-`)
static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\S+ - \S+ \[\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2} [+\-]\d{4}\] ""#)
        .expect("invalid detect regex")
});

// Trailing key=value pairs (e.g. rt=0.003 uct=0.001)
static KV_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(\w+)=([\S]+)"#).expect("invalid kv regex"));

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

pub struct NginxParser;

impl NginxParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NginxParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for NginxParser {
    fn name(&self) -> &'static str {
        "nginx"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = NGINX_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match Nginx access log format".into()))?;

        let remote_addr = caps.get(1).map(|m| m.as_str()).unwrap_or("-");
        let remote_user = caps.get(2).map(|m| m.as_str()).unwrap_or("-");
        let timestamp_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        let request = caps.get(4).map(|m| m.as_str()).unwrap_or("");
        let status_str = caps.get(5).map(|m| m.as_str()).unwrap_or("0");
        let bytes_str = caps.get(6).map(|m| m.as_str()).unwrap_or("-");
        let referer = caps.get(7).map(|m| m.as_str()).unwrap_or("-");
        let user_agent = caps.get(8).map(|m| m.as_str()).unwrap_or("");
        let trailing = caps.get(9).map(|m| m.as_str()).unwrap_or("");

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
        if referer != "-" {
            attributes.insert(
                "http.referer".to_string(),
                serde_json::Value::String(referer.to_string()),
            );
        }
        attributes.insert(
            "user_agent.original".to_string(),
            serde_json::Value::String(user_agent.to_string()),
        );

        // Parse trailing key=value pairs (nginx extensions like rt=, uct=, etc.)
        for kv_cap in KV_RE.captures_iter(trailing) {
            let key = kv_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let val = kv_cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let attr_key = match key {
                "rt" => "nginx.request_time",
                "uct" => "nginx.upstream_connect_time",
                "uht" => "nginx.upstream_header_time",
                "urt" => "nginx.upstream_response_time",
                other => other,
            };
            attributes.insert(
                attr_key.to_string(),
                serde_json::Value::String(val.to_string()),
            );
        }

        Ok(UniversalEvent {
            timestamp,
            hostname: None,
            service: Some("nginx".to_string()),
            severity: Some(status_to_severity(status).to_string()),
            message: format!("{} → {}", request, status),
            raw_event: raw.to_string(),
            network: Some(NetworkInfo {
                source_ip: Some(remote_addr.to_string()),
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
            description: "Nginx Access Log Parser (combined + extended formats)".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STANDARD: &str = r#"192.168.1.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326 "http://example.com/" "Mozilla/5.0 (X11; Linux x86_64)""#;

    const EXTENDED: &str = r#"10.0.0.1 - - [28/Aug/2026:10:30:00 +0000] "POST /api/ingest HTTP/1.1" 201 45 "-" "python-requests/2.28.0" rt=0.003 uct=0.001 urt=0.002"#;

    const NOT_FOUND: &str = r#"203.0.113.50 - - [28/Aug/2026:12:00:00 +0000] "GET /missing.html HTTP/1.1" 404 169 "-" "Googlebot/2.1""#;

    const BAD_GATEWAY: &str = r#"172.16.0.10 - - [28/Aug/2026:12:05:00 +0000] "GET /upstream HTTP/1.1" 502 0 "-" "curl/7.68.0""#;

    #[test]
    fn test_detect_standard() {
        let parser = NginxParser::new();
        assert!(parser.detect(STANDARD));
    }

    #[test]
    fn test_detect_extended() {
        let parser = NginxParser::new();
        assert!(parser.detect(EXTENDED));
    }

    #[test]
    fn test_detect_rejects_syslog() {
        let parser = NginxParser::new();
        assert!(!parser.detect("<34>Oct 11 22:14:15 mymachine su: test"));
    }

    #[test]
    fn test_parse_standard() {
        let parser = NginxParser::new();
        let event = parser.parse(STANDARD).unwrap();

        assert_eq!(event.severity.unwrap(), "INFO");
        assert_eq!(event.message, "GET /index.html HTTP/1.1 → 200");
        assert_eq!(event.service.unwrap(), "nginx");

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
    }

    #[test]
    fn test_parse_extended_with_timing() {
        let parser = NginxParser::new();
        let event = parser.parse(EXTENDED).unwrap();

        assert_eq!(event.severity.unwrap(), "INFO");
        assert_eq!(
            event.attributes.get("nginx.request_time").unwrap(),
            &serde_json::Value::String("0.003".into())
        );
        assert_eq!(
            event.attributes.get("nginx.upstream_connect_time").unwrap(),
            &serde_json::Value::String("0.001".into())
        );
        assert_eq!(
            event
                .attributes
                .get("nginx.upstream_response_time")
                .unwrap(),
            &serde_json::Value::String("0.002".into())
        );
    }

    #[test]
    fn test_parse_404_is_warn() {
        let parser = NginxParser::new();
        let event = parser.parse(NOT_FOUND).unwrap();
        assert_eq!(event.severity.unwrap(), "WARN");
    }

    #[test]
    fn test_parse_502_is_error() {
        let parser = NginxParser::new();
        let event = parser.parse(BAD_GATEWAY).unwrap();
        assert_eq!(event.severity.unwrap(), "ERROR");
    }
}
