//! # OpenSSH Log Parser
//!
//! Parses OpenSSH authentication and server logs:
//!
//! ```text
//! Dec 10 06:55:46 LabSZ sshd[24200]: reverse mapping checking getaddrinfo for ns.marryaldkfaczcz.com [173.234.31.186] failed - POSSIBLE BREAK-IN ATTEMPT!
//! ```

use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use helios_core::{Error, ProcessInfo, Result, UniversalEvent};
use helios_parser::{Parser, ParserMetadata};
use regex::Regex;
use std::sync::LazyLock;

static DETECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}\s+\S+\s+sshd(?:\[\d+\])?:\s*")
        .expect("invalid detect regex")
});

static CAPTURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^([A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(\S+)\s+sshd(?:\[(\d+)\])?:\s*(.*)$",
    )
    .expect("invalid capture regex")
});

fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    let year = Utc::now().year();
    let normalized_ts = ts.split_whitespace().collect::<Vec<_>>().join(" ");
    let full_ts = format!("{} {}", year, normalized_ts);
    NaiveDateTime::parse_from_str(&full_ts, "%Y %b %d %H:%M:%S")
        .ok()
        .map(|dt| dt.and_utc())
}

fn infer_severity(msg: &str) -> String {
    let lower = msg.to_lowercase();
    if lower.contains("fail")
        || lower.contains("error")
        || lower.contains("break-in attempt")
        || lower.contains("invalid user")
        || lower.contains("disconnecting:")
        || lower.contains("fatal:")
    {
        "WARN".to_string()
    } else {
        "INFO".to_string()
    }
}

pub struct OpenSshParser;

impl OpenSshParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenSshParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for OpenSshParser {
    fn name(&self) -> &'static str {
        "openssh"
    }

    fn detect(&self, raw: &str) -> bool {
        DETECT_RE.is_match(raw)
    }

    fn parse(&self, raw: &str) -> Result<UniversalEvent> {
        let caps = CAPTURE_RE
            .captures(raw)
            .ok_or_else(|| Error::ParseError("Does not match OpenSSH log format".into()))?;

        let timestamp_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let host = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let pid_str = caps.get(3).map(|m| m.as_str());
        let message = caps.get(4).map(|m| m.as_str()).unwrap_or("");

        let timestamp = parse_timestamp(timestamp_str).unwrap_or_else(Utc::now);
        let pid = pid_str.and_then(|p| p.parse::<u32>().ok());

        Ok(UniversalEvent {
            timestamp,
            hostname: Some(host.to_string()),
            service: Some("sshd".to_string()),
            severity: Some(infer_severity(message)),
            message: message.to_string(),
            raw_event: raw.to_string(),
            process: Some(ProcessInfo {
                pid,
                name: Some("sshd".to_string()),
                thread_id: None,
                thread_name: None,
            }),
            ..Default::default()
        })
    }

    fn metadata(&self) -> ParserMetadata {
        ParserMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "OpenSSH authentication and service log parser".to_string(),
            author: "Helios Team".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str =
        "Dec 10 06:55:46 LabSZ sshd[24200]: reverse mapping checking getaddrinfo for ns.marryaldkfaczcz.com [173.234.31.186] failed - POSSIBLE BREAK-IN ATTEMPT!";
    const ACCEPTED_SAMPLE: &str =
        "Dec 10 09:32:20 LabSZ sshd[24680]: Accepted password for root from 192.168.1.5 port 51234 ssh2";

    #[test]
    fn test_detect() {
        let parser = OpenSshParser::new();
        assert!(parser.detect(SAMPLE));
        assert!(parser.detect(ACCEPTED_SAMPLE));
    }

    #[test]
    fn test_detect_rejects_other() {
        let parser = OpenSshParser::new();
        assert!(!parser.detect(r#"{"message": "just json"}"#));
    }

    #[test]
    fn test_parse() {
        let parser = OpenSshParser::new();
        let event = parser.parse(SAMPLE).unwrap();
        assert_eq!(event.hostname.as_deref(), Some("LabSZ"));
        assert_eq!(event.service.as_deref(), Some("sshd"));
        assert_eq!(event.severity.as_deref(), Some("WARN"));
        assert_eq!(
            event.message,
            "reverse mapping checking getaddrinfo for ns.marryaldkfaczcz.com [173.234.31.186] failed - POSSIBLE BREAK-IN ATTEMPT!"
        );
        assert_eq!(event.raw_event, SAMPLE);

        let proc = event.process.unwrap();
        assert_eq!(proc.pid, Some(24200));
        assert_eq!(proc.name.as_deref(), Some("sshd"));
    }

    #[test]
    fn test_parse_accepted() {
        let parser = OpenSshParser::new();
        let event = parser.parse(ACCEPTED_SAMPLE).unwrap();
        assert_eq!(event.severity.as_deref(), Some("INFO"));
        assert_eq!(event.hostname.as_deref(), Some("LabSZ"));
    }
}
