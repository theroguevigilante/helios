//! # Proxifier Log Parser
//!
//! Parses Proxifier client proxy logs:
//!
//! ```text
//! [10.30 16:49:06] chrome.exe - proxy.cse.cuhk.edu.hk:5070 open through proxy proxy.cse.cuhk.edu.hk:5070 HTTPS
//! ```

use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use helios_core::{Error, ProcessInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[\d{2}\.\d{2} \d{2}:\d{2}:\d{2}\] \S.* - ").expect("invalid detect regex")
});

static CAPTURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\[(\d{2}\.\d{2} \d{2}:\d{2}:\d{2})\] (.*?) - (.*)$")
        .expect("invalid capture regex")
});

fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    let year = Utc::now().year();
    let full_ts = format!("{}.{}", year, ts);
    NaiveDateTime::parse_from_str(&full_ts, "%Y.%m.%d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
}

fn infer_severity(msg: &str) -> String {
    let lower = msg.to_lowercase();
    if lower.contains("error") || lower.contains("failed") || lower.contains("could not connect") {
        "ERROR".to_string()
    } else {
        "INFO".to_string()
    }
}

pub struct ProxifierParser;

impl ProxifierParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProxifierParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for ProxifierParser {
    fn name(&self) -> &'static str {
        "proxifier"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = CAPTURE_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match Proxifier log format".into()))?;

        let timestamp_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let program = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let message = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        let timestamp = parse_timestamp(timestamp_str).unwrap_or_else(Utc::now);

        Ok(UniversalEvent {
            timestamp,
            hostname: None,
            service: Some(program.to_string()),
            severity: Some(infer_severity(message)),
            message: message.to_string(),
            raw_event: raw.to_string(),
            process: Some(ProcessInfo {
                pid: None,
                name: Some(program.to_string()),
                thread_id: None,
                thread_name: None,
            }),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Proxifier proxy client log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str =
        "[10.30 16:49:06] chrome.exe - proxy.cse.cuhk.edu.hk:5070 open through proxy proxy.cse.cuhk.edu.hk:5070 HTTPS";
    const ERROR_SAMPLE: &str =
        "[10.30 17:15:42] QQ.exe - tcpconn6.tencent.com:443 error : A connection request was canceled before the completion.";

    #[test]
    fn test_detect() {
        let parser = ProxifierParser::new();
        assert!(parser.detect(SAMPLE));
        assert!(parser.detect(ERROR_SAMPLE));
    }

    #[test]
    fn test_detect_rejects_other() {
        let parser = ProxifierParser::new();
        assert!(!parser.detect(r#"{"message": "just json"}"#));
    }

    #[test]
    fn test_parse() {
        let parser = ProxifierParser::new();
        let event = parser.parse(SAMPLE).unwrap();
        assert_eq!(event.service.as_deref(), Some("chrome.exe"));
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(
            event.message,
            "proxy.cse.cuhk.edu.hk:5070 open through proxy proxy.cse.cuhk.edu.hk:5070 HTTPS"
        );
        assert_eq!(event.raw_event, SAMPLE);

        let proc = event.process.unwrap();
        assert_eq!(proc.name.as_deref(), Some("chrome.exe"));
    }

    #[test]
    fn test_parse_error() {
        let parser = ProxifierParser::new();
        let event = parser.parse(ERROR_SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("ERROR"));
        assert_eq!(event.service.as_deref(), Some("QQ.exe"));
    }
}
